label chapter9:
    #第二幕第四场
    ###清晨的教室##
    scene bg gaozhongjiaoshi_h with Dissolve(1.0)
    play music "audio/BGM/1.ensolarado.mp3"
    男生A "喂，你们也来听听啊，这是不是红叶配的音啊。"
    男生B "这也太像了吧，感觉根本就是本音吧。"
    女生A "啊？"
    女生B "这是红叶的新作吗…也没有听到什么消息啊。"
    男生B "你这是从哪里搞来的啊？"
    男生A "这就是社团大作业评比的成果啊，你们自己去图书馆那边看嘛。听完快把耳机还我…喂，不要抢啊…"
    女生B "说起来，这个故事也很有意思呢。这几个人设太好玩了。"
    show hongye summer_normal2 at s31 
    旁白 "我和红叶走进教室。"
    红叶 summer_normal2"嗯？"
    旁白 "众人退散，回到各自的座位上等待早读课开始。"
    女生A "为什么这两个人身上的洗衣液味道一模一样啊？该不会——"
    众人 "唔——"
    子文 "啥？"
    红叶 summer_backhand3"咳——"
    旁白 "众人安静。"
    hide hongye
    #
    ###图书馆##
    scene bg tushuguanneijing_h with tran_black_anticlockwise()
    show hongye summer_normal1 at s22
    红叶 summer_normal1 "哎子文，我们的广播剧，似乎确实大受好评呢。"
    子文 "那可不，红叶大小姐本音出演，那可不是要迷倒一片了。我也被迷倒了。"
    红叶 summer_backhand1 "不过你看，似乎评论板对剧本也赞赏有加呢。"
    旁白 "评论版上写着：“剧本好精彩，我也被感动了。”“这个配音和这个剧本简直绝配啊！”“这是哪个工作室的单品啊，我单推了。”"
    hide hongye
    子文 "唔…"
    红叶 summer_backhand2 "嗯？"
    子文 "这个剧本是谁写的呢，真是羡慕嫉妒他啊。"
    红叶 summer_backhand3 "你这话说的…"
    子文 "行了，快走吧，最后一节阅读摘抄课了，珍惜吧。"
    stop music
    旁白 "..."
    show fengdao a at s21
    #TODO:这段旁白感觉有点奇怪
    旁白 "丰岛走到了展出区域。"
    旁白 "在学生基本走光了之后，他开始逐一查看各稿件。然后注意到了《音与言语的即兴剧》。"
    丰岛 c "广播剧？"
    旁白 "丰岛戴起了公用电脑的耳机。"
    scene black with dissolve
    
    ###子文的公寓，晚饭时间##
    scene bg ziwendegongyu_y with dissolve
    play music "audio/BGM/2.昼下がり気分.mp3"
    子文 "喔，这可是红叶大小姐的手艺，我可得带着敬畏之心下筷子啊。"
    show hongye summer_normal5 at s11
    红叶 summer_normal5"少废话赶快吃饭，今天的任务还堆积如山呢。"
    hide hongye with dissolve
    
    ###晚上，书桌旁##
    show hongye pajamas_backhand3 at s22("close")
    红叶 pajamas_backhand3"你怎么连这个都搞不清楚啊，上次就做错过的题目，这次还错是吧？"
    子文 "别骂了别骂了。"
    hide hongye

    ###教室##
    scene bg gaozhongjiaoshi_h with tran_black_close()
    show hongye summer_normal2 at s11
    红叶 summer_normal2 "子文，你上哪去。回来学习，今天作业写完了吗？"
    子文 "呃，我就转转。"
    #这个感觉怪怪的
    红叶 summer_backhand3 "你还有时间转转啊？"
    hide hongye

    ###班主任办公室##
    scene bg banzhurenbangongshi_a with tran_black_close()
    班主任 "你这张卷子做的整体还行，但是错了几个基础题，还是得好好背。"
    show hongye summer_backhand2 at s22()
    旁白 "我点头。手边是高考考纲内古诗文全集。"
    scene bg banzhurenbangongshi_a
    show hongye summer_normal5 at s22(easein_time = 0)
    #TODO:这个离开效果我回头做
    pause 0.5
    hide hongye with dissolve
    ##演出 "红叶笑一笑，然后离开##"
    stop music
    
    scene bg gaozhongjiaoshi_y with tran_black_close()
    旁白 "一周后。"
    show hongye summer_backhand4 at s22
    旁白 "放学之后，我们准备离开。这时班主任走进教室找到我们。"
    班主任 "二位，有些事情，请过来一下。"
    hide hongye with dissolve
    ###办公室##
    scene bg banzhurenbangongshi_y with tran_black_anticlockwise()
    旁白 "红叶的父母都在。"
    show hongye summer_normal2 at s11
    红叶 summer_normal2"…"
    show hongye summer_normal2 at s22
    班主任 "关于红叶同学的事情，为了不耽误她高考，也不耽误她的事业，今天也许可以作一个了结。"
    红叶 summer_backhand2 "呵…"
    班主任 "首先是一个结果。你可以按照自己的意愿去报考声优学校了，落英小姐。"
    红叶 summer_backhand4 "啊？"
    旁白 "红叶看起来非常惊讶。"
    班主任 "说来很巧，那天我看到有同事在阅读娱乐新闻，刚好涉及“湖城玉樱赏”的颁奖。虽然这已经是旧闻，但是我一眼看见了其中有一张照片里的人有点眼熟。"
    班主任 "于是我就去借来了这张报纸，后来请两位家长也确认了一下，无疑照片里的人就是红叶。"
    红叶 summer_normal2 "你们认错了吧，我怎么会出现在这种场合。"
    班主任 "红叶，如果是以我三年对同学们的熟悉程度，或许只是眼熟而已，但是你的父母亲是绝对不会认错你的啊。"
    play music "audio/BGM/4.終幕への間奏曲.mp3"
    红叶母 "傻孩子，你倒是早点说啊。我们也是刚刚才知道，你以“落英”名义出演的作品，这几年你的努力，不仅在学校吃苦，还有这么多作品…"
    红叶 summer_normal6 "不…"
    红叶母 "你再怎么化妆，我和爸爸也能一眼认出我们的骨肉。我看见了你站在领奖台上的笑容，那是这些年在家里我们从来没有见过的。"
    红叶母 "我的孩子，即便是面对父母也要化妆...我是多么失败的母亲啊..."
    红叶母 "如果声优对你来说是这样快乐的事情，我和爸爸肯定是会支持的啊。"
    红叶 summer_normal6 "… "
    红叶母 "傻孩子，这几年我和你爸工作都紧张，没时间和你聊，也没有注意这些事情。我还有爸爸在这里给你道歉。"
    红叶母 "但是，千万不要再一句话不说就跑出去了，以后什么都可以给爸妈讲。你逃走的那天晚上，真的…"
    红叶 summer_backhand6 "…"
    旁白 "我和班主任站在稍远的地方。"
    hide hongye
    子文 "看来其实我没有发挥什么作用啊…"
    班主任 "并非如此。哪有人真的心肠如铁石的。少年，如果有必要的话，以后你也要当好她坚强的支柱啊。"
    子文 "啥？"
    #TODO:班主任目送子文和红叶他们走出去。
    #夕阳从窗户斜射进来。
    #TODO:###给一个夕阳的特写##
    stop music
    班主任 "啊，青春真好啊。"
    旁白 "..."
    scene bg huanghun with dissolve
    班主任 "要是，那时…"
    play music "audio/BGM/8.紫苑_-追憶-.mp3"
    scene bg tushuguanmenqian_y with dissolve
    show fengdao a at s21
    pause
    scene bg tushuguanneijing_y with dissolve
    pause
    hide fengdao 
    scene bg ziwendegongyu_y with dissolve
    旁白 "我藏起了我的作文纸，还有我的摘抄本。"
    子文 "下次再见是什么时候呢…"
    scene bg yekong with dissolve
    pause 1.0
    stop music
    scene black with dissolve
    
     
