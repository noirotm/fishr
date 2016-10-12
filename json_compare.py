import json
import sys

def get_lines(filename):
    with open(filename) as f:
        return f.readlines()

pylines = get_lines("py.log")
rlines = get_lines("r.log")

for i in range(0, min(len(pylines), len(rlines))):
    o1 = json.loads(pylines[i])
    o2 = json.loads(rlines[i])

    if o1 != o2:
        print "line %d:\n%s\n!=\n%s" % (i+1, o1, o2)
        sys.exit(0)