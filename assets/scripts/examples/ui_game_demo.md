# 新功能最小示例

> 说明：本脚本用于手动回归 `textMode` / `showMap` / `requestUI` / `callGame`。
> 说明：当前 `callGame` 仍处于骨架阶段，会走空字符串降级路径，不会卡住脚本。

**start**

changeScene <img src="../../backgrounds/white.png" style="zoom:10%;" /> with Fade(duration: 0.5)

："新增功能最小示例开始。"

："第一部分：切换到 NVL 模式。"
textMode nvl

："这里是 NVL 模式第一行。"
："这里是 NVL 模式第二行。"

pause

textMode adv
："现在已经回到 ADV 模式。"

："第二部分：测试地图语法糖 showMap。"
showMap "demo_world" as $destination

if $destination == "beach"
  ："你通过 showMap 选择了海边。"
elseif $destination == "town"
  ："你通过 showMap 选择了小镇。"
else
  ："你通过 showMap 选择了其他地点。"
endif

："第三部分：直接测试底层 requestUI。"
requestUI "show_map" as $raw_destination (map_id: "demo_world")

if $raw_destination == "forest"
  ："你通过 requestUI 直接选择了森林。"
elseif $raw_destination == "beach"
  ："你通过 requestUI 直接选择了海边。"
else
  ："你通过 requestUI 直接选择了其他地点。"
endif

："第四部分：测试 callGame 小游戏。"
callGame "demo_stub" as $game_result

if $game_result == "win"
  ："恭喜！你在小游戏中获胜了！"
elseif $game_result == "lose"
  ："很遗憾，小游戏挑战失败了。下次加油！"
elseif $game_result == ""
  ："callGame 返回空字符串，可能 WebView 不可用。"
else
  ："callGame 返回了意外结果。"
endif

："示例结束。"
