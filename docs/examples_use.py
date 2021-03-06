import binascii
import examples_pb2

def to_hex(data):
    """Convert bytes to a hex string"""
    return binascii.hexlify(data).decode('utf8')

child1 = examples_pb2.Person()
child1.id = 959435311
child1.name = "Susanne Doe"
child1.age = 34
child1.weight = 70.02
child1.mood = -20

grandchild = examples_pb2.Person()
grandchild.id = 95941545151
grandchild.name = "Jane Smith"
grandchild.age = 1.3
grandchild.weight = 12.657
grandchild.mood = 15

child2 = examples_pb2.Person()
child2.id = 81154811
child2.name = "Mac Smith"
child2.children.append(grandchild)
child2.age = 27.04
child2.weight = 80.6662
child2.mood = 0

person = examples_pb2.Person()
person.id = 1021211
person.name = "John Doe"
person.children.append(child1)
person.children.append(child2)
person.age = 56.1
person.weight = 65.12
person.mood = 75

print(grandchild.name + ":")
serialization = grandchild.SerializeToString()
with open('grandchild.bin', 'wb') as w:
    w.write(serialization)
print(to_hex(serialization))

print("")

print(person.name + ":")
serialization = person.SerializeToString()
with open('person.bin', 'wb') as w:
    w.write(serialization)
print(to_hex(serialization))

# Use this command to generate binaries:
# python3 ./examples_use.py
