# Ring Remake 主入口（vNext 草案）

> 说明：本文件使用“Ring Script vNext（设想语法）”组织跨文件流程。  
> 章节正文已搬迁至 `assets/scripts/remake/summer/*.rpy` 与 `assets/scripts/remake/winter/*.rpy`。  
> 正在翻译中的 Ring 版章节位于 `assets/scripts/remake/ring/`。  
> 当前运行时尚未支持 `callScript` / `returnFromScript`，因此本入口属于“理论最终运行脚本组”的总调度稿。

**Summer**
callScript "ring/summer/prologue.md" as **prologue**
callScript "ring/summer/1-1.md" as **chapter1**
callScript "ring/summer/1-2.md" as **chapter2**
callScript "ring/summer/1-3.md" as **chapter3**
callScript "ring/summer/1-4.md" as **chapter4**
callScript "ring/summer/1-5.md" as **chapter5**
callScript "ring/summer/2-1.md" as **chapter6**
callScript "ring/summer/2-2.md" as **chapter7**
callScript "ring/summer/2-3.md" as **chapter8**
callScript "ring/summer/2-4.md" as **chapter9**
callScript "ring/summer/2-5.md" as **chapter10**
callScript "ring/summer/3-1.md" as **chapter11**

if $complete_summer != true
  set $complete_summer = true
  scene bg:black with dissolve(0.5)
  callScript "summer/splash.rpy" as **splashscreen**
  fullRestart
else
  goto **Winter**
endif

**Winter**
callScript "ring/winter/inter01.md" as **chapter12**
callScript "ring/winter/3-2.md" as **chapter13**
callScript "ring/winter/3-3.md" as **chapter14**
callScript "ring/winter/inter02.md" as **chapter15**
callScript "ring/winter/3-4.md" as **chapter16**
callScript "ring/winter/3-5.md" as **chapter17**
callScript "ring/winter/3-6.md" as **chapter18**
callScript "ring/winter/inter03.md" as **chapter19**
callScript "ring/winter/3-7.md" as **chapter20**
callScript "ring/winter/ending.md" as **ending**
end
