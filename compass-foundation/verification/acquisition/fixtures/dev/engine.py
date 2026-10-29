"""Independent development controls; these modules are never executed."""
def leaf(value):
    return value + 7  # DEV_LEAF_LITERAL

def caller(value):
    return leaf(value)

class One:
    def same(self):
        return "one"

class Two:
    def same(self):
        return "two"

def unresolved(function, value):
    return function(value)
