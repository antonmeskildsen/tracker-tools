import ascc


if __name__ == "__main__":

    exp = ascc.load_asc_from_file("/home/anton/Documents/work/viscom-tools/test_files/SmoothPursuits_128_2.asc")

    print(len(exp.trials))