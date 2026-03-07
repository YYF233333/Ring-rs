# Ring Remake 主入口（vNext 草案）

> 说明：本文件使用“Ring Script vNext（设想语法）”组织跨文件流程。  
> Ring 语义章节已迁移至 `assets/scripts/remake/ring/`（summer 12 章 + winter 10 章）。  
> 原始 `summer/*.rpy` 与 `winter/*.rpy` 仅保留用于对照与回归参考。  
> 以上说明为注释文本；脚本解析阶段应跳过 `>` 开头行。  
> 当前跨文件流程使用 `callScript [label](path)`；非入口文件到末尾自动 return，入口文件到末尾自动结束并返回主界面。

**Summer**
callScript [prologue](ring/summer/prologue.md)
callScript [chapter1](ring/summer/1-1.md)
callScript [chapter2](ring/summer/1-2.md)
callScript [chapter3](ring/summer/1-3.md)
callScript [chapter4](ring/summer/1-4.md)
callScript [chapter5](ring/summer/1-5.md)
callScript [chapter6](ring/summer/2-1.md)
callScript [chapter7](ring/summer/2-2.md)
callScript [chapter8](ring/summer/2-3.md)
callScript [chapter9](ring/summer/2-4.md)
callScript [chapter10](ring/summer/2-5.md)
callScript [chapter11](ring/summer/3-1.md)

if $complete_summer != true
  set $complete_summer = true
  scene bg:black with dissolve(0.5)
  callScript [splashscreen](summer/splash.rpy)
  fullRestart
else
  goto **Winter**
endif

**Winter**
callScript [chapter12](ring/winter/inter01.md)
callScript [chapter13](ring/winter/3-2.md)
callScript [chapter14](ring/winter/3-3.md)
callScript [chapter15](ring/winter/inter02.md)
callScript [chapter16](ring/winter/3-4.md)
callScript [chapter17](ring/winter/3-5.md)
callScript [chapter18](ring/winter/3-6.md)
callScript [chapter19](ring/winter/inter03.md)
callScript [chapter20](ring/winter/3-7.md)
callScript [ending](ring/winter/ending.md)
end
