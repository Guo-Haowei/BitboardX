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

    # TODO: parse score
    if sys.argv[2] == 'stockfish':
        engine2 = 'stockfish1600'
        elo = 1600
        engine2_setup = [
            '-engine', f'name={engine2}',
            f'cmd=stockfish', 'option.UCI_LimitStrength=true', f'option.UCI_Elo={elo}'
        ]
    else:
        engine2 = get_file_name(sys.argv[2])
        engine2_setup = ['-engine', f'name={engine2}', f'cmd=./releases/{engine2}.exe']

    run_command([
        cubechess_cli,
        '-engine', f'name={engine1}', f'cmd=./releases/{engine1}.exe',
        *engine2_setup,
        '-each',
        'proto=uci',
        'tc=40/60',
        '-rounds', '50',
        '-pgnout', f'{engine1}-vs-{engine2}.pgn',
        # '-debug', 'all'
    ])

main()