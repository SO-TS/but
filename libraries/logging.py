import sys
import time

start_time = 0

def print(*values: object, sep: str = "", end: str = "\n"):
    if start_time == 0:
        return 0
    string = ""
    for i in values:
        string = string + sep + str(i)
    string = string + "\n"
    with open(f"./logs/asd-{int(start_time)}.log", "a+", encoding="utf-8") as f:
        f.write(f"[{'{:.5f}'.format(time.time() - start_time)}] {string}")
    sys.stdout.write(f"[{'{:.5f}'.format(time.time() - start_time)}] {string}")
    