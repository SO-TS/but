import libraries.logging as log
from libraries.zip import zip_folder as backup
import json
import sys
import os
import time

# but，文件夹备份工具。
# by @Stevesuk0.
# 该工具是 WSMCS Server 运维程序的一部分。

version = "0.1"
sys.argv.pop(0)
argv = sys.argv
os.makedirs("logs", exist_ok=True)

def about():
    print(f"but {version}，文件夹备份工具。\n")
    print(f"@Stevesuk0，版本：{version}")

def init_config():
    config = {
        "backup": [
            {
                "name": "test",
                "from": "path/to/folder",
                "dest": "./"
            }
        ],
        "settings": {
            "interval": 300,
            "saving_name": "%name%-%timestamp%"
        }
    }
    with open("but.json", "w") as f:
        f.write(json.dumps(config, indent=4))

def load_config():
    try:
        if not os.path.isfile("but.json"):
            init_config()
            print("配置文件生成完毕，请重新执行 but。")
            exit()
        with open("but.json") as f:
            return json.loads(f.read())
    except Exception as f:
        print("but: 配置文件无法加载，原因为：")
        print(f)
        exit()

def start_listen():
    lastTime = time.time()
    log.start_time = lastTime
    log.print("开始备份监听。")
    while True:
        config = load_config()
        time.sleep(1)
        if time.time() - lastTime > config["settings"]["interval"]:
            for i in range(len(config["backup"])):        
                log.print(f"[{i+1}/{len(config['backup'])}] 正在备份 {config["backup"][i]["name"]}")        
                backupName = config["settings"]["saving_name"]
                backupName = backupName.replace("%name%", config["backup"][i]["name"])
                backupName = backupName.replace("%timestamp%", str(int(time.time())))
                backup(config["backup"][i]["from"], f"{config["backup"][i]["dest"]}{backupName}.zip")
            lastTime = time.time()
            log.print("本次备份全部完成。")
        

def help():
    print("""but - 用法:
          
配置操作:
    -i, -g,  --init         生成配置文件
          
无参数启动将开始直接备份。
    
如果您是第一次使用，请输入 \"but --init\" 生成配置文件。
请对配置文件进行修改来使用 but，后续更新将添加更多功能。
不留参数将会开始文件备份。""")


if len(argv) == 0:
    start_listen()
elif argv[0] in ('--version', '-v', '--about'):
    about()
elif argv[0] in ('-i', '-g', '--init'):
    init_config()
else:
    help()
