from entry import dispatch_value

def run_batch(amounts):
    return [dispatch_value(amount) for amount in amounts]
