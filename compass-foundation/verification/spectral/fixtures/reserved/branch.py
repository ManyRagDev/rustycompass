from policy import capacity_rule

def route_adjustment(amount):
    left = capacity_rule(amount)
    right = capacity_rule(amount + 2)
    return max(left, right)
