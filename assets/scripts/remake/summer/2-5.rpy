label chapter10:
    #第二幕第五场
    scene bg ziwenjia_n with dissolve
    旁白 "6月6日晚上。已经是高考前一天了。"
    旁白 "我暂时回到了父母家里住着，因为他们说要给我安排什么高考食谱，不允许我再自己乱吃东西。"
    #TODO:（优先级不高）这个转场可以改良，旧背景滑动移出
    scene bg ziwenzijiadefangjian_snowkin with slideleft
    旁白 "红叶打来了电话。"
    红叶 pajamas_normal1 "再检查一遍包啊，看一下准考证。身上有什么金属制品，像手表什么的，都提前弄下来，安检很严格的。"
    子文 "知道了…"
    红叶 pajamas_backhand1 "提前去看过考场了吗？虽然就在本校考试，但是不在高三的教学楼吧。"
    子文 "看过了…"
    红叶 pajamas_backhand2 "文具袋准备好了吗？标签记得撕掉，要全透明的。再看一遍有没有带其他违禁的东西，可千万不要在这里出问题啊。"
    子文 "我说，你怎么跟我妈一样。"
    红叶 pajamas_normal3 "因为担心子文进不去考场啊。你这个月以来一直埋着头看书，也不怎么和我打电话，我怕你变成呆子。"
    子文 "开什么玩笑，你把我当什么了。反倒是你啊，虽然已经预录取了，但是也要好好考试的吧，可不要玩过头了。"
    红叶 pajamas_backhand3 "你这话说的，什么叫玩啊，你以为我就不用工作了吗。是不是只要每天确认自己不是哑巴，然后录音的时候往话筒前面一站，就能和事务所签约了啊？（笑）"
    旁白 "我笑了。"
    红叶 pajamas_backhand1 "好了好了，准备早点休息吧。"
    子文 "嗯。晚安。"
    红叶 pajamas_normal4"啊，还有一个事情…"
    子文 "嗯？"
    play music "audio/BGM/14-ED1.野良猫は宇宙を目指した.mp3"
    红叶 pajamas_backhand2"就是…那个…"
    红叶 pajamas_normal5"…"
    红叶 pajamas_backhand1"算了，考完试再给你说吧。"
    子文 "什么事情啊，整的这么神秘。"
    红叶 pajamas_normal4 "考完试你应该没有别的事情了吧？"
    子文 "9号考完之后，我想想，隔天还要去学校把丢在储存室的书都搬回来。然后应该就没啥事情了。"
    红叶 pajamas_normal1 "那就好。"
    子文 "所以说到底是啥事啊？"
    红叶 pajamas_backhand5 "你快睡觉吧。考完你就知道了。"
    旁白 "..."
    
    旁白 "房间门被敲响了。"
    子文母 "儿子，你评评理，你爸说我给你弄的高考菜单菜太少了，你说少不少？"
    子文 "呃…"
    子文父 "哎呀我讲了这几天我来做饭…"
    子文母 "我绝对不答应。这几天的状态一定要稳定，伙食上有任何一点不确定因素都会导致不可预测的结果。"
    子文 "啊，我觉得好像差不多啊…"
    子文母 "啊呀啊呀。儿子你先休息吧。你爸的问题我得再和他比划比划…"
    旁白 "还是赶快睡觉吧。"

    ###给几个时间变化的镜头##
    scene bg gaozhongjiaoshi_h with tran_black_anticlockwise()
    pause 
    scene bg jiaoshizoulang_y with wiperight
    pause
    scene bg tushuguanmenqian_y with tran_close()
    pause
    scene bg jiangchengjiejing_y with tran_df()
    pause
    scene bg louwaijingyewan_n with multigraph([Solid("#000")])

    stop music
    旁白 "在考完最后一门的当晚。"
    ###外景##
    show hongye summer_normal5 at s11
    红叶 "好热啊…湖城的夏天总是如此呢。"
    子文 "所以说…"
    红叶 summer_backhand4 "嗯？"
    子文 "你之前说的事情…"
    红叶 summer_backhand1 "我可没有忘哦。"
    旁白 "她拿出了两张火车票。"
    hide hongye with dissolve
    子文 "6月11日，湖城到…江城？"
    红叶 summer_normal1"对！这是毕业旅行。去江城，看玉樱节！"
    scene black with dissolve
     