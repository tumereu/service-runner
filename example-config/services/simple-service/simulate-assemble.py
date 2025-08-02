import time
import random
import os


def simulate_assemble():
    print()
    print("Assembling artifacts..", flush=True)
    time.sleep(random.uniform(1, 3))

    os.makedirs("build", exist_ok=True)
    artifact_path = os.path.join("build", "build-artifact.example")

    with open(artifact_path, "w") as f:
        f.write("Example build results")

    print(f"Created: {artifact_path}")

if __name__ == "__main__":
    simulate_assemble()
