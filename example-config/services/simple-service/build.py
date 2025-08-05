import time
import random

def print_with_delay(text, min_delay=0.01, max_delay=0.2):
    print(text, flush=True)
    time.sleep(random.uniform(min_delay, max_delay))

def simulate_gradle_build():
    start_time = time.time()

    print_with_delay("> Task :compileJava")
    print_with_delay("> Task :processResources")
    print_with_delay("> Task :classes")
    print_with_delay("> Task :compileTestJava")
    print_with_delay("> Task :processTestResources")
    print_with_delay("> Task :testClasses")
    print_with_delay("> Task :test")
    print_with_delay("Test run started...")
    print_with_delay("Executing tests...")
    print_with_delay("Test successful.")
    print_with_delay("> Task :jar")
    print_with_delay("> Task :assemble")
    print_with_delay("> Task :check")
    print_with_delay("> Task :build")

    print()
    print("BUILD SUCCESSFUL in {:.2f}s".format(time.time() - start_time))
    print("7 actionable tasks: 7 executed")

if __name__ == "__main__":
    simulate_gradle_build()
