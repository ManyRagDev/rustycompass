"""Cancel a booking and release the reserved room for another visitor."""
def cancel_booking(reservations, key):
    return reservations.pop(key)
