import rust_extension

class ThatThing:
    def __init__(self, q):
        print("ThatThing")
        self.q = q

    def helo(self):
        print(f"world {self.q}")

    def doit(selfs):
        rust_extension.hello_from_rust()