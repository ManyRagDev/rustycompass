from policy import base_rule

def apply_fee(amount):
    adjusted = base_rule(amount)
    return adjusted + 7
