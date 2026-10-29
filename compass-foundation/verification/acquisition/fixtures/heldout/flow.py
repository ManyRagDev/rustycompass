def finish(item):
    return item * 3  # HOLD_LEAF_LITERAL

def start(item):
    return finish(item)

class Alpha:
    def repeat(self):
        return 1

class Beta:
    def repeat(self):
        return 2

def deferred(callback):
    return callback(4)
