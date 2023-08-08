import ascc


if __name__ == "__main__":

    exp = ascc.load_asc_from_file("resources/SmoothPursuits_180_1.asc")

    print(len(exp.trials))