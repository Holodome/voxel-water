import matplotlib.pyplot as plt

with open("docs/inc/timing.csv") as f:
    data = []
    for line in f.readlines():
        a, b, c = line.split(",")
        t = (int(a), int(b), float(c))
        data.append(t)


def plot_slice(a, t):
    items = [i for i in data if i[1] == a]
    plt.plot([i[0] for i in items], [i[2] for i in items], label=t)


plot_slice(1, "число отскоков 1")
plot_slice(2, "число отскоков 2")
plot_slice(4, "число отскоков 4")
plot_slice(8, "число отскоков 8")
plt.ylabel("время, мкс")
plt.xlabel("дальность видимости")
plt.legend(loc="upper left")
plt.savefig("result.svg")

# plt.show()
