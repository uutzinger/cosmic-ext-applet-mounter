"""Disposable fake Git/GitHub/package CLIs. Never contact real services."""

import json
import os
from pathlib import Path
import shutil
import subprocess

ROOT = Path(__file__).resolve().parents[1]

FAKE = r'''#!/usr/bin/env python3
import hashlib,json,os,pathlib,sys,time
name=pathlib.Path(sys.argv[0]).name
args=sys.argv[1:]
root=pathlib.Path(os.environ['FIXTURE_ROOT'])
cfg=json.loads((root/'config.json').read_text())
with (root/'commands.jsonl').open('a') as log: log.write(json.dumps([name,*args])+'\n')
def save(): (root/'config.json').write_text(json.dumps(cfg))
def emit(value): print(json.dumps(value))
def run():
 return {'id':123,'repository':{'full_name':'example/project'},'head_repository':{'full_name':'example/project'},
         'head_sha':cfg.get('run_sha','a'*40),'path':cfg.get('workflow_path','.github/workflows/build-deb.yml'),
         'event':'workflow_dispatch','status':cfg.get('status','completed'),'conclusion':cfg.get('conclusion','success')}
def package(folder,v,a):
 folder.mkdir(parents=True,exist_ok=True)
 p=folder/f'cosmic-ext-applet-mounter_0.5.4_ubuntu{v}_{a}.deb'
 p.write_bytes(b'fixture package')
 (folder/'SHA256SUMS').write_text(hashlib.sha256(p.read_bytes()).hexdigest()+'  '+p.name+'\n')
if cfg.get('sleep_command')==name:
 (root/'child.pid').write_text(str(os.getpid()))
 time.sleep(60)
if name=='git':
 if args[0]=='branch': print('master')
 elif args[0]=='status': print(' M unrelated' if cfg.get('dirty') else '',end='')
 elif args[0]=='remote': print('git@github.com:example/project.git')
 elif args[0]=='rev-parse': print(cfg.get('local_tag_sha','a'*40) if args[-1].startswith('refs/tags/') else 'a'*40)
 elif args[0]=='show-ref': sys.exit(0 if cfg.get('local_tag') else 1)
 elif args[0]=='ls-remote':
  if cfg.get('remote_exit'): sys.exit(cfg['remote_exit'])
  if any(a.startswith('refs/heads/') for a in args): print('a'*40+'\trefs/heads/master')
  elif cfg.get('remote_tag'): print(cfg.get('remote_tag_sha','a'*40)+'\trefs/tags/v0.5.4^{}')
 elif args[0]=='tag': cfg['local_tag']=True;save()
 elif args[0]=='push':
  if cfg.get('push_exit'): sys.exit(cfg['push_exit'])
  cfg['remote_tag']=True;save()
 elif args[0]=='fetch': cfg['local_tag']=True;save()
 else: sys.exit(91)
elif name=='cargo': emit({'packages':[{'name':'cosmic-ext-applet-mounter','version':'0.5.4'}]})
elif name=='dpkg-parsechangelog': print('0.5.4')
elif name=='dpkg-deb':
 field=args[-1]
 print({'Package':'cosmic-ext-applet-mounter','Version':'0.5.4','Architecture':'arm64' if 'arm64' in args[-2] else 'amd64'}[field])
elif name=='gh':
 if args[:2]==['repo','view']: print('example/project')
 elif args[0]=='api':
  endpoint=args[-1]
  if '/commits/' in endpoint: emit({'sha':cfg.get('remote_source','a'*40)})
  elif '/git/ref/tags/' in endpoint: emit({'object':{'type':'commit','sha':'a'*40}})
  elif '/artifacts?' in endpoint:
   emit({'total_count':6,'artifacts':[{'name':f'deb-ubuntu-{v}-{a}','expired':False} for v in ['22.04','24.04','26.04'] for a in ['amd64','arm64']]})
  elif '/actions/runs/' in endpoint: emit(run())
  else: sys.exit(92)
 elif args[:2]==['workflow','run']:
  cfg['request_id']=next(a.split('=',1)[1] for a in args if a.startswith('request_id='));save()
  print('NOISY-DISPATCH\n'*1000,file=sys.stderr)
  sys.exit(cfg.get('dispatch_exit',0))
 elif args[:2]==['run','list']:
  title='Debian packages · '+cfg.get('request_id','legacy')
  items=[{'databaseId':99,'headSha':'a'*40,'event':'workflow_dispatch','displayTitle':'unrelated-old-run'}]
  if not cfg.get('stale_only'):
   items.append({'databaseId':123,'headSha':'a'*40,'event':'workflow_dispatch','displayTitle':title})
  if cfg.get('ambiguous'): items.append(dict(items[-1],databaseId=124))
  emit(items)
 elif args[:2]==['run','download']:
  print('NOISY-DOWNLOAD\n'*1000,file=sys.stderr)
  if cfg.get('download_exit'): sys.exit(cfg['download_exit'])
  folder=pathlib.Path(args[args.index('--dir')+1])
  if '--name' in args:
   n=args[args.index('--name')+1]; _,_,v,a=n.split('-');package(folder,v,a)
  else:
   for v in ['22.04','24.04','26.04']:
    for a in ['amd64','arm64']: package(folder/f'deb-ubuntu-{v}-{a}',v,a)
 elif args[:2]==['release','create']: print('https://example.invalid/release')
 else: sys.exit(93)
else: sys.exit(94)
'''


class ToolingFixture:
    def __init__(self, root):
        self.root = Path(root)
        self.config = self.root / "config.json"
        self.config.write_text("{}")
        (self.root / "scripts").mkdir()
        for name in ["run_github_linux_build.sh", "build_release.sh", "promote_release.py", "check-release-publication.sh"]:
            shutil.copy2(ROOT / "scripts" / name, self.root / "scripts" / name)
        shutil.copytree(ROOT / "scripts/lib", self.root / "scripts/lib", ignore=shutil.ignore_patterns("__pycache__"))
        fake = self.root / "fake-cli"
        fake.write_text(FAKE)
        fake.chmod(0o755)
        for name in ["git", "gh", "cargo", "dpkg-deb", "dpkg-parsechangelog"]:
            (self.root / name).symlink_to(fake)
        self.env = os.environ.copy()
        for key in ["DOWNLOAD_DIR", "WORKFLOW", "REF"]:
            self.env.pop(key, None)
        self.env.update(PATH=f"{self.root}:{self.env['PATH']}", FIXTURE_ROOT=str(self.root))

    def configure(self, **fields):
        state = json.loads(self.config.read_text())
        state.update(fields)
        self.config.write_text(json.dumps(state))

    def run(self, script="run_github_linux_build.sh", *args):
        return subprocess.run(["bash", str(self.root / "scripts" / script), "--poll-seconds", "1", *args],
                              cwd=self.root, env=self.env, capture_output=True, text=True, timeout=15)

    def commands(self):
        path = self.root / "commands.jsonl"
        return [json.loads(line) for line in path.read_text().splitlines()] if path.exists() else []

    def summary(self, result):
        return json.loads(result.stdout.splitlines()[-1])
