from boto_ext_qbusiness_chat_stream import QBusiness


b = QBusiness()

import asyncio

async def async_iterable(values):
    for value in values:
        yield value

async def main():
    turn = b.prepare_chat("account", "app", "user")
    events = async_iterable(["apple", "banana", "cherry"])
    result = await turn.send_chat(events)
    print(result)

asyncio.run(main())