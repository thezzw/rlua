local short_string = "A short string."
local middle_string = "This is a string that is a bit longer."
local long_string = "This is a long string that is longer than the middle one."

dbg_print(short_string)
dbg_print(middle_string)
dbg_print(long_string)

print("tab:\t-") -- tab
print("null: \0.") -- '\0'
print("\xE4\xBD") -- invalid UTF-8
print("\72\101\108\108\111") -- Hello
print("\xE7\xAB\xB9\xE7\x9F\xA5\xE5\x90\xBE\x0D") -- 竹知吾
