label debug:
    "开发用测试章节"
    $ music_class = channel_class()
    $ music_class.time = 0
    $ fading_pause(filename = "在りし日のために -inst ver- - 松本文紀_01.flac", fade = 0.5, pause = None, channel_class = music_class, volume = 0.25)
    "淡出暂停示例"
    "Click to pause."
    $ fading_pause(filename = "在りし日のために -inst ver- - 松本文紀_01.flac", fade = 0.5, channel_class = music_class, volume = 0.25)
    "Click to unpause."
    $ fading_pause(filename = "在りし日のために -inst ver- - 松本文紀_01.flac", fade = 0.5, pause = None, channel_class = music_class, volume = 0.25)
    "即将更换音量"
    $ fading_pause(filename = "在りし日のために -inst ver- - 松本文紀_01.flac", fade = 0.0, channel_class = music_class, volume = 1.0)
    $ fading_pause(filename = "在りし日のために -inst ver- - 松本文紀_01.flac", fade = 0.0, pause = None, channel_class = music_class, volume = 1.0)


# # refer to https://lemmasoft.renai.us/forums/viewtopic.php?t=43406

#     class channel_class(NoRollback):
#         def __init__(self):
#             self.time = 0

#     def _insert(name, time, channel_class):#将暂停的音乐的信息计入字典
#         channel_class.time = time
            
#     def fading_pause(channel = "music", pause = "toggle", fade = 1.0, filename = None, channel_class=None):
#         if pause == "toggle":
#             pause = renpy.music.get_playing(channel)#获取当前channel中播放的音乐，没有则返回None
        
#         if pause:
#             _insert(renpy.music.get_playing(), renpy.music.get_pos() + fade / 2, channel_class)
#             renpy.music.stop(channel, fade)
#         else:
#             # if filename == None:
#             #     name, time = pauseddict.popitem()
#             # else:
#             #     if filename in pauseddict.keys():
#             #         name, time = filename, pauseddict[filename]
#             #     else:
#             #         name, time = filename, 0
#             # fn = "<from {}>".format(time) + name
#             fn = "<from {}>".format(channel_class.time) + filename
#             renpy.music.play(fn, channel, fadein = fade)

# # example for using:
# # 1.
# # label start:
# #     "Start"
# #     play music "music.mp3"
# #     "Click to pause."
# #     $ fading_pause()
# #     "Click to unpause."
# #     $ fading_pause()
# #     "End"
# #     return
# # 2.
# # $ fading_pause(channel = "music", pause = True, fade = 1.0, filename = "Song.mp3")

# # label gallery:
# #
# #     if not persistent.gallery_unlocked:
# #         show background
# #         centered "你还没有解锁画廊。"
# #         $ renpy.full_restart()
# #
# #     # 这里实际展示画廊。
# #
# # $ persistent.gallery_unlocked = True

# # 合并持久化数据
# #
# # init python:
# #     if persistent.endings is None:
# #         persistent.endings = set()

# #     def merge_endings(old, new, current):
# #         current.update(old)
# #         current.update(new)
# #         return current

# #     renpy.register_persistent('endings', merge_endings)

# # 不回滚

# # init python:

# #     class MyClass(NoRollback):
# #         def __init__(self):
# #             self.value = 0

# # label start:
# #     $ o = MyClass()

# #     "欢迎！"

# #     $ o.value += 1

# #     "o.value的值是 [o.value] 。你每次回滚并点到这里都会增加它的值。"