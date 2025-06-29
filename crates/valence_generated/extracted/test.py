import json
import pprint

stats  = {}

with open("items.json") as file:
    data = json.load(file)
    for item in data:
        for j in item["components"]:
            j = j.replace(":", ".")
            if j not in stats:
                stats[j] = 1
            else:
                stats[j] += 1
                
pprint.pp(stats)