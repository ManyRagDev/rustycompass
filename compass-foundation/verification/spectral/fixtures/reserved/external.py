import os

def external_choice(amount, callback):
    return callback(amount) + int(os.environ["REMOTE_FEE"])
