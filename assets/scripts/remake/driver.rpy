# 游戏在此开始。
label Summer:
    # "debug" #TODO:最后记得删除
    call prologue
    call chapter1
    call chapter2
    call chapter3
    call chapter4
    call chapter5
    call chapter6
    call chapter7
    call chapter8
    call chapter9
    call chapter10
    call chapter11
    if not persistent.complete_summer:
        $ persistent.complete_summer = True
        scene black with dissolve
        with Pause(0.5)
        call splashscreen
        $ renpy.full_restart(transition=None)
    else:
        return

label Winter:
    call chapter12
    call chapter13
    call chapter14
    call chapter15
    call chapter16
    call chapter17
    call chapter18
    call chapter19
    call chapter20
    call ending
    return
    
#default last_chapter = ""
#
#default chapters = {}
#
#menu:
#    "prologue":
#        jump prologue
#    "1-1":
#        jump chapter1
#    "1-2":
#        jump chapter2
#    "1-3":
#        jump chapter3
#    "1-4":
#        jump chapter4
#    "1-5":
#        jump chapter5
#    "2-1":
#        jump chapter6
#    "2-2":
#        jump chapter7
#    "2-3":
#        jump chapter8
#    "2-4":
#        jump chapter9
#    "2-5":
#        jump chapter10
#    "3-1":
#        jump chapter11
#    "3-2":
#        jump chapter13
#    "3-3":
#        jump chapter14
#    "3-4":
#        jump chapter16
#    "3-5":
#        jump chapter17
#    "3-6":
#        jump chapter18
#    "3-7":
#        jump chapter20
#    "inter01":
#        jump chapter12
#    "inter02":
#        jump chapter15
#    "inter03":
#        jump chapter19
#    "ending":
#        jump ending
