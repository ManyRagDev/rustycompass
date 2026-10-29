from branch import route_adjustment
from policy import capacity_rule

def dispatch_value(amount):
    if amount < 0:
        return capacity_rule(0)
    return route_adjustment(amount)

def local_tag(amount):
    return amount // 2
