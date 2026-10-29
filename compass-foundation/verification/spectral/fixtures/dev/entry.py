from bridge import apply_fee

def invoice_total(amount):
    return apply_fee(amount)

def local_tag(amount):
    return "local:" + str(amount)
