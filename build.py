import subprocess
import shutil
from pathlib import Path

def main():
    try:
        print('building Rust WebAssembly...')
        static_dir = Path('src/static')
        for block in Path('wasm-blocks').iterdir():
            print(f'runing wasm-pack for block {block.name}...')
            subprocess.check_call(['wasm-pack', 'build', '--target', 'web'], cwd=block)
            print('copying generated WASM to static dir...')
            target_dir = static_dir / 'wasm' / block.name
            target_dir.mkdir(parents=True, exist_ok=True)
            for item in (block / 'pkg').iterdir():
                if item.suffix not in ['.wasm', '.js']:
                    continue
                dest = target_dir / item.name
                if item.is_dir():
                    shutil.copytree(item, dest, dirs_exist_ok=True)
                else:
                    shutil.copy2(item, dest)
        print('completed Rust WebAssembly build')
        
        print('install dependencies...')
        subprocess.check_call(['npm', 'ci'], shell=True)

        print('setup site target directory...')
        subprocess.check_call(['mkdir', '-p', 'gen'])
        
        print('build static 11ty site...')
        subprocess.check_call(['npm', 'run', 'build'], shell=True)

    except subprocess.CalledProcessError as e:
        print(f'error during build: {e}')
        exit(1)

if __name__ == '__main__':
    main()
