from .q_stream_rs import say_hello

class ThatThing:
    def __init__(self, q):
        print("ThatThing")
        self.q = q

    def helo(self):
        print(f"world {self.q}")

    def doit(self):
        say_hello()