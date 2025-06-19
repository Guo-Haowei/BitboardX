import subprocess
import shutil
import sys
import os

def main():
    if len(sys.argv) < 2:
        print('Usage: python build.py <new_executable_name>')
        sys.exit(1)

    # Step 1: Build the Rust project
    print('Building Rust project with `cargo build --release`...')
    result = subprocess.run(['cargo', 'build', '--release'])
    if result.returncode != 0:
        print('Cargo build failed.')
        sys.exit(1)

    # Step 2: Rename the executable
    target_dir = os.path.join('target', 'release')
    release_folder = 'releases'
    exe_name = f'BitboardX_{sys.argv[1]}'
    exe_path = os.path.join(target_dir, 'uci.exe')
    new_exe_path = os.path.join(release_folder, f'{exe_name}.exe')
    print(f'Renaming executable from {exe_path} to {new_exe_path}...')
    if os.path.exists(new_exe_path):
        os.remove(new_exe_path)
    shutil.move(exe_path, new_exe_path)
    print(f'Executable renamed to {new_exe_path}')

main()