import subprocess
import shutil
import sys
import os

def get_file_name(file_path):
    base_name = os.path.basename(file_path)
    name_only = os.path.splitext(base_name)[0]
    return name_only

def run_command(command):
    print(f'Running command: {' '.join(command)}')
    result = subprocess.run(command, capture_output=True, text=True)
    if result.returncode != 0:
        print('Command failed:')
        print(result.stdout)
        print(result.stderr)
        sys.exit(1)
    return

def main():
    if len(sys.argv) != 3:
        print('Usage: python match.py <engine_name1> <engine_name2>')
        sys.exit(1)

    cubechess_cli = 'cutechess-cli.exe'
    engine1 = get_file_name(sys.argv[1])
    engine2 = get_file_name(sys.argv[2])

    run_command([
        cubechess_cli,
        '-engine', f'name={engine1}', f'cmd=./releases/{engine1}.exe',
        '-engine', f'name={engine2}', f'cmd=./releases/{engine2}.exe',
        '-each',
        'proto=uci',
        'tc=inf',
        # 'tc=40/60',
        '-rounds', '100',
        '-pgnout', f'{engine1}-vs-{engine2}.pgn',
        '-debug', 'all'
    ])

main()