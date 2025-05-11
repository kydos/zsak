import requests

ws = key_expr.split('/')
city = ws[-1]

url = f"https://wttr.in/{city}?2F"
response = requests.get(url)
if response.ok:
    result = response.text
else:
    result = "Don't have any weather info on " + city
