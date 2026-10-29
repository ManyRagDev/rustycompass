def outer():
    def inside():
        return "DEV_WRAPPER_LITERAL"
    return inside()
