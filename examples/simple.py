import ascc


if __name__ == "__main__":

    exp = ascc.load_experiment_file("resources/Patterns_214_0.dat")

    print(len(exp.trials))