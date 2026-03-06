# 第一章：功能测试

："欢迎来到 Visual Novel Engine 功能测试！"

："首先测试背景切换（无过渡效果）..."

changeBG <img src="../backgrounds/BG12_pl_n_19201440.jpg" style="zoom:25%;" >

stopBGM

："背景已切换，这是白天场景。"

<audio src="../sfx/ding.mp3"></audio>

## 1.1 背景过渡测试

："接下来测试 dissolve 过渡效果..."

changeBG <img src="../backgrounds/BG12_pl_cy_19201440.jpg" style="zoom:25%;" > with Dissolve(1.0)

："这是黄昏场景，使用了 dissolve 淡入淡出过渡。"

："接下来测试 changeScene 的 Fade 复合过渡（黑屏遮罩 + 清立绘 + 换背景）..."

changeScene <img src="../backgrounds/cg1.jpg" style="zoom:15%;" > with Fade(duration: 1)

："现在切换到 CG 场景，使用了 Fade 黑屏复合过渡。"

："测试 changeScene 的 FadeWhite 复合过渡（白屏遮罩 + 清立绘 + 换背景）..."

changeScene <img src="../backgrounds/BG12_pl_n_19201440.jpg" style="zoom:25%;" > with FadeWhite(duration: 1)

："回到白天场景，使用了 FadeWhite 白屏复合过渡。"

："测试 changeScene 的 rule 遮罩复合过渡（图片遮罩）..."

changeScene <img src="../backgrounds/BG12_pl_cy_19201440.jpg" style="zoom:15%;" > with <img src="../backgrounds/rule_10.png" style="zoom:15%;" alt="rule_10" /> (duration: 1, reversed: true)

："回到黄昏场景，使用了 rule 图片遮罩复合过渡。"

："背景过渡测试完成！"

### 1.1.1 角色显示测试

："现在测试角色显示功能..."

北风："大家好，我是北风！"

show <img src="../characters/北风-日常服.png" style="zoom:25%;" > as beifeng at center

北风："我现在出现在画面中央。"

："测试角色位置：左侧"

show beifeng at left with dissolve

北风："我移动到了左侧。"

："测试角色位置：右侧"

show beifeng at right with dissolve

北风："我现在在右侧了。"

："测试角色位置：近"

show beifeng at nearmiddle

北风："我现在出现在画面中央。"

："测试角色位置：左侧"

show beifeng at nearleft

北风："我移动到了左侧。"

："测试角色位置：右侧"

show beifeng at nearright

北风："我现在在右侧了。"

："测试角色位置：远"

show beifeng at farmiddle

北风："我现在出现在画面中央。"

："测试角色位置：左侧"

show beifeng at farleft

北风："我移动到了左侧。"

："测试角色位置：右侧"

show beifeng at farright

北风："我现在在右侧了。"

："测试角色切换立绘"

show <img src="../characters/北风-日常服2.png" style="zoom:25%;" > as beifeng at center with dissolve

北风："我换了一套衣服！使用了 dissolve 过渡效果。"

#### 选择分支测试

北风："接下来要测试选择分支功能。"

："请选择你想要测试的内容："

| 测试选项 |  |
| --- | --- |
| 测试旁白文本 | test_narration |
| 测试对话变体 | test_dialogue |
| 测试脚本逻辑（变量/表达式/条件） | test_logic |
| 跳过，直接结束 | test_end |

**test_narration**

："这是一段旁白文本。"

："旁白可以描述场景、角色的内心活动等内容。"

："旁白测试完成，跳转到结束部分..."

goto **test_end**

**test_dialogue**

北风："这是普通对话。"

？？？："神秘人物也可以说话！"

北风："对话测试完成。"

**test_logic**

："接下来测试脚本逻辑系统：变量 / 表达式 / 条件分支。"

："先设置一些变量。"

set $player_name = "Alice"
set $role = "user"
set $has_key = false
set $door_locked = true
set $a = true
set $b = false
set $score = 42

："测试 if/elseif/else："

if $role == "admin"
  ："分支：admin（不应出现）"
elseif $role == "user"
  ："分支：user（应出现）"
else
  ："分支：else（不应出现）"
endif

："测试 not / and / or / 括号："

if not $has_key and ($door_locked == true)
  ："门是锁着的，而且你没有钥匙（应出现）"
else
  ："这句不应出现"
endif

if ($a == true) and ($b == false)
  ："括号与逻辑与：($a==true) and ($b==false)（应出现）"
endif

if ($a == false) or ($b == false)
  ："逻辑或：($a==false) or ($b==false)（应出现）"
endif

："测试字符串比较与不等比较："

if $player_name != "Bob"
  ："player_name != Bob（应出现）"
endif

："测试 set 写入表达式结果："

set $is_ok = $a == true and $b == false
if $is_ok == true
  ："is_ok == true（应出现）"
endif

："测试 Int："

if $score == 42
  ："score == 42（应出现）"
endif

："脚本逻辑测试完成，返回结束部分..."
goto **test_end**

**test_end**

北风："现在测试角色隐藏功能..."

hide beifeng with dissolve

："北风已经离开画面了。"

# 第二章：测试完成

："恭喜！所有基础功能测试完成！"

："感谢测试！"
