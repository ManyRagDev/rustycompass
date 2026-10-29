def enclosing():
    def enclosed():
        return "HOLD_WRAPPER_LITERAL"
    return enclosed()
