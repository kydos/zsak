import requests

ws = key_expr.split('/')
word = ws[-1]

url = f"https://api.dictionaryapi.dev/api/v2/entries/en/{word}"
response = requests.get(url)
print(response)
if response.status_code == 200:
    data = response.json()    
    result = ""
    meanings = data[0]["meanings"]
    for meaning in meanings:
        part_of_speech = meaning["partOfSpeech"]
        definitions = meaning["definitions"]
        for definition in definitions:
            result = result + f"\n({part_of_speech}) {definition['definition']}"                    

else:
    result = str("I don't know this word")

