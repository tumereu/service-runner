import time
import random
import os
import shutil

def simulate_clean():
    if os.path.isdir("build"):
        shutil.rmtree("build")

    print("Clean complete", flush=True)

if __name__ == "__main__":
    simulate_clean()
