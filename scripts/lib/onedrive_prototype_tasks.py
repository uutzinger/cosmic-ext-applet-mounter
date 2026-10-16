"""Logged fixed-action driver for the development-only Rust OneDrive prototype."""
import argparse
from pathlib import Path
import sys
from task_output import TaskReport

ACTIONS = ('auth', 'inspect', 'prepare', 'initial-sync', 'verify', 'scenario', 'benchmark', 'timing',
           'fault-network', 'fault-cancel', 'repair', 'cleanup')


def parse(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--action', required=True, choices=ACTIONS)
    parser.add_argument('--remote', required=True)
    parser.add_argument('--drive', required=True, choices=('personal', 'business'))
    parser.add_argument('--expected-account', required=True,
                        help='Expected Microsoft sign-in identity, checked in memory; not recorded in logs')
    parser.add_argument('--transport', choices=('native', 'flatpak-host'), default='native')
    parser.add_argument('--session', help='Absolute retained session path from a previous prepare receipt')
    parser.add_argument('--files-dir', help='Absolute sample directory, copied read-only; prepare only')
    parser.add_argument('--log-dir')
    args = parser.parse_args(argv)
    if args.files_dir and (args.action != 'prepare' or not Path(args.files_dir).is_absolute() or not Path(args.files_dir).is_dir()):
        parser.error('--files-dir requires prepare and an existing absolute directory')
    if args.action not in ('auth', 'inspect', 'prepare') and not args.session:
        parser.error('--session is required for this action')
    if args.session and (args.action in ('auth', 'inspect', 'prepare') or not Path(args.session).is_absolute()):
        parser.error('--session must be absolute and used only for a retained session action')
    if not args.remote or len(args.remote) > 96 or any(not (c.isascii() and (c.isalnum() or c in '_-.')) for c in args.remote):
        parser.error('Invalid remote name')
    return args


def main(argv=None):
    args = parse(argv)
    with TaskReport('onedrive-prototype', args.log_dir) as report:
        sha = report.capture('git', 'rev-parse', 'HEAD')
        dirty = bool(report.capture('git', 'status', '--porcelain'))
        report.update(source_sha=sha, source_dirty=dirty, operation=args.action,
                      transport=args.transport, artifact_receipt_path=str(report.directory / 'prototype.json'))
        # Quiet suppresses Cargo's echoed process arguments, including the expected account.
        command = ['cargo', 'run', '--quiet', '--locked', '--features', 'development-probes', '--example',
                   'onedrive_mirror_prototype', '--', '--action', args.action, '--remote', args.remote,
                   '--drive', args.drive, '--expected-account', args.expected_account, '--transport',
                   args.transport, '--report-dir', str(report.directory)]
        if args.session:
            command.extend(['--session', args.session])
        if args.files_dir:
            command.extend(['--files-dir', args.files_dir])
        report.phase(args.action, next_action='Read prototype.json and retained session.json; live acceptance is action-specific.')
        report.run(command, timeout=3600)
        report.phase('action-complete')


if __name__ == '__main__':
    main(sys.argv[1:])
