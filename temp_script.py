import os
def get_dirty():
    import subprocess
    cp = subprocess.run(['git', 'status', '--porcelain', '-uall'], capture_output=True, text=True)
    return cp.stdout.splitlines()

dirty = get_dirty()
print('Dirty entries count:', len(dirty))
with open('/tmp/dirty.txt', 'w') as out:
    for d in dirty:
        out.write(d + '
')
