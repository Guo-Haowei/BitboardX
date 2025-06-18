import os
import zipfile
import subprocess

def build_and_zip(project_dir):
    dist_path = os.path.join(project_dir, 'dist')
    out_path = os.path.join(project_dir, 'chess')

    print(f'Changing directory to: {project_dir}')
    os.chdir(project_dir)

    print('Running: npm run build')
    result = subprocess.run([r'C:\Program Files\nodejs\npm.cmd', 'run', 'build'], capture_output=True, text=True)

    if result.returncode != 0:
        print('Build failed:')
        print(result.stdout)
        print(result.stderr)
        return

    os.chdir('..')

    if not os.path.exists(dist_path):
        print(f'Build succeeded, but "{dist_path}" folder not found.')
        return

    # print(f'renaming "{dist_path}" to "{out_path}"')
    # os.rename(dist_path, out_path)

    print(f'Done!')

build_and_zip('frontend')