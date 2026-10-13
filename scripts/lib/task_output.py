"""Private durable task receipts and bounded console reporting (standard library)."""

import datetime
import hashlib
import json
import os
from pathlib import Path
import signal
import subprocess
import tempfile
import traceback


class Cancelled(BaseException):
    def __init__(self, signum):
        self.code = 128 + signum


def atomic_json(path, value):
    path = Path(path)
    fd, temporary = tempfile.mkstemp(prefix=".receipt-", dir=path.parent)
    try:
        with os.fdopen(fd, "w") as stream:
            json.dump(value, stream, indent=2)
            stream.write("\n")
        os.replace(temporary, path)
    finally:
        if os.path.exists(temporary):
            os.unlink(temporary)


class TaskReport:
    def __init__(self, task, log_dir=None):
        if log_dir:
            self.directory = Path(log_dir).absolute()
            self.directory.mkdir(mode=0o700, parents=True, exist_ok=False)
        else:
            base = Path("target/task-runs")
            base.mkdir(parents=True, exist_ok=True)
            self.directory = Path(tempfile.mkdtemp(prefix=task + "-", dir=base)).absolute()
        self.log_path = self.directory / "task.log"
        self.log = self.log_path.open("x")
        self.log_path.chmod(0o600)
        self.state = dict.fromkeys(["source_sha", "ref", "repository", "run_id", "run_url",
                                    "request_id", "exit_code", "artifact_receipt_path"])
        self.state.update(schema_version=1, task=task, status="running", phase="preflight",
                          log_path=str(self.log_path), summary_path=str(self.directory / "summary.json"),
                          next_action="Inspect the task log if this operation does not complete.")
        self.handlers = {}
        self.update()

    def update(self, **fields):
        self.state.update(fields)
        self.state["updated_at"] = datetime.datetime.now(datetime.timezone.utc).isoformat()
        atomic_json(self.directory / "summary.json", self.state)

    def phase(self, name, **fields):
        changed = self.state["phase"] != name
        self.update(phase=name, **fields)
        if changed:
            self.log.write(f"\n[phase: {name}]\n")
            self.log.flush()
            print(f"{self.state['task']}: {name}", flush=True)

    def run(self, argv, capture=False, allowed=(0,), timeout=120):
        # Do not echo arbitrary arguments: auth/file paths can be sensitive.
        self.log.write(f"\n[command: {Path(argv[0]).name}]\n")
        self.log.flush()
        with tempfile.TemporaryFile() as stdout:
            process = subprocess.Popen(argv, stdout=stdout if capture else self.log,
                                       stderr=self.log, start_new_session=True)
            try:
                code = process.wait(timeout=timeout)
            except BaseException:
                if process.poll() is None:
                    try:
                        os.killpg(process.pid, signal.SIGTERM)
                    except ProcessLookupError:
                        pass
                    try:
                        process.wait(timeout=5)
                    except subprocess.TimeoutExpired:
                        try:
                            os.killpg(process.pid, signal.SIGKILL)
                        except ProcessLookupError:
                            pass
                        process.wait()
                raise
            if capture:
                stdout.seek(0)
                data = stdout.read(8 * 1024 * 1024 + 1)
                self.log.write(data.decode(errors="replace"))
                self.log.flush()
                if len(data) > 8 * 1024 * 1024:
                    raise ValueError("Command metadata exceeded the capture budget")
            if code not in allowed:
                raise subprocess.CalledProcessError(code, [Path(argv[0]).name])
            return data.decode().strip() if capture else code

    def capture(self, *argv, **kwargs):
        return self.run(list(argv), capture=True, **kwargs)

    def artifacts(self, packages, path):
        files = [{"path": str(p.absolute()), "sha256": hashlib.sha256(p.read_bytes()).hexdigest()}
                 for p in packages]
        atomic_json(path, {"schema_version": 1, "source_sha": self.state["source_sha"],
                           "run_id": self.state["run_id"], "artifacts": files})
        self.update(artifact_receipt_path=str(Path(path).absolute()))

    def __enter__(self):
        def interrupted(signum, _frame):
            raise Cancelled(signum)
        for signum in (signal.SIGINT, signal.SIGTERM):
            self.handlers[signum] = signal.signal(signum, interrupted)
        return self

    def __exit__(self, kind, error, tb):
        if isinstance(error, Cancelled):
            code, status = error.code, "cancelled"
        elif isinstance(error, subprocess.CalledProcessError):
            code = error.returncode if error.returncode > 0 else 128 - error.returncode
            status = "failed"
        elif error:
            code, status = 1, "failed"
        else:
            code, status = 0, "succeeded"
        if error:
            traceback.print_exception(kind, error, tb, file=self.log)
        self.update(status=status, exit_code=code)
        self.log.close()
        for signum, handler in self.handlers.items():
            signal.signal(signum, handler)
        print(json.dumps({key: self.state[key] for key in ["task", "status", "exit_code",
                         "source_sha", "run_id", "summary_path", "log_path",
                         "artifact_receipt_path", "next_action"]}), flush=True)
        if error:
            raise SystemExit(code)
        return False
