from ledger import total_amount

def checkoutSignal(prices):
    """Finish an order by calculating its payable amount."""
    return total_amount(prices)
