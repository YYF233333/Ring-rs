label chapter8:
    #第二幕第三场
    scene bg gaozhongjiaoshi_y with dissolve
    旁白 "六点，晚上放学时间，教室里已经不剩下几个人。"
    旁白 "我收拾好了包，像往常一样收拾好包去找红叶。"
    show hongye summer_normal6 at s22
    子文 "红叶，走吧。"
    红叶 summer_normal6 "抱歉，我有点别的事。你先回去吧。"
    子文 "没事吧？你这是…"
    红叶 summer_backhand2 "我没事，你快回去吧。"
    子文 "你昨天没睡，今天又是一天正课，这哪是像没事的样子？要么在这里休息一下吧，或者去医务室？我背着你也行的…"
    show hongye summer_normal2 at s22(easein_time = 0.05)
    红叶 summer_normal2 "{cps=*2}我没事！不要管我！{/cps}" with hpunch
    子文 "…"
    play music "audio/BGM/10.星粒が降る夜.mp3"
    旁白 "我后退一步。"
    show hongye summer_normal2 at s11("far")
    红叶 summer_normal6"抱，抱歉。你先回去吧。"
    子文 "那…千万千万不要勉强自己。"
    红叶 summer_backhand2 "… "
    hide hongye with dissolve
    scene bg jiaoshizoulang_y with dissolve
    旁白 "我走出教室，贴着门口的墙，仔细听着教室里的声响。"
    旁白 "{cps=*0.2}…{/cps}"
    旁白 "大概过了十分钟，教室里没有一丁点声音。红叶仍然坐在她的位置上。"
    scene black with tran_lf()
    pause 0.3
    scene bg jijiangxiayu with dissolve
    旁白 "我还是决定回家。"
    旁白 "今天开始要一心复习了。"
    #TODO:不知道改啥特效好
    scene bg xiaomaibu_ with dissolve
    旁白 "在小卖部购买了一盒方便面，一个茶叶蛋，一包纯牛奶，囫囵塞到包里。"
    旁白 "冰箱里应该还剩下一些东西，但是我也懒得自己做饭了。"
    ###场景切换到路上##
    #TODO:只是个占位
    scene black with tran_lf()
    pause 0.3
    scene bg tongxuelu_n with dissolve
    play sound "<from .2>audio/SE/刹车.mp3"
    骑车人 "喂，小屁孩，走路不长眼睛的吗？" with vpunch
    ###刹车声##
    旁白 "一辆自行车几乎擦着我过去。"
    子文 "抱歉，抱歉，实在不好意思…"
    旁白 "这时我才注意到，天空像覆了一层的水泥。今天大概要下大雨吧。"
    scene bg ziwendegongyu_nn with tran_black_anticlockwise()
    stop music
    ###子文的公寓##
    旁白 "我收起了原先摆在桌上杂乱无章的草稿纸和作文纸，拿出了没有怎么动过的教科书和参考书。"
    旁白 "大约晚上九点，突然门铃响了。"
    旁白 "父母临时来访吗，为什么没有电话通知我。"
    show hongye summer_normal6 at s11("far")
    旁白 "我没怎么想地开了门。红叶穿着白天的校服，背着书包站在门口，全身湿透了。"
    play music "audio/BGM/11.ある冬の寒い夜に.mp3"
    show hongye summer_normal6 at s11
    红叶 summer_normal6"…"
    #旁白 "泫然欲泣的表情。"
    子文 "红…叶？你为什么…"
    show hongye summer_backhand6 at s11("close")
    旁白 "我立刻把红叶拉进屋子，关起门。"
    红叶 summer_normal2 "如你所见，我离家出走了。接下来可能要住在这里一段时间。"
    子文 "到底怎么回事？"
    红叶 summer_backhand2 "我妈，班主任，年级主任，还有几个老师，把我堵在教室里，要我当面签字结束与事务所的合同，以及放弃报考声优学校。"
    红叶  "于是我逃走了。我撞开了他们，然后跑到了街上。天突然变得很暗，并且开始下大雨。这也好，阻止了他们追上我。我在城里转了几圈，回过神来已经到你家门口了。"
    旁白 "她打了一个喷嚏。"
    stop music 
    子文 "先别说这个了，赶快去洗澡换一身衣服。呃，红叶，你有带衣服吗？"
    红叶 summer_normal2 "我可是有在你这里预存了不少东西的哦。"
    子文 "那...行。架子上的毛巾都是干净的随便用…"
    hide hongye with dissolve
    scene bg ziwengongyuchufang_y with tran_black_anticlockwise()
    旁白 "我走进厨房，打开冰箱，发现只剩下了几个番茄，土豆，还有鸡蛋。垃圾桶里有晚上的泡面盒。"
    旁白 "呼，好吧。还是要做一顿饭。我姑且对自己做番茄蛋汤和炒土豆丝的水平还是有自信的。"
    #TODO:SE 做饭的时候哼歌（建议wzy实录哼哼哼啊啊啊啊啊）
    #FIXME:哼歌好，臭爪。不过哼什么呢
    旁白 "…"
    scene bg ziwendegongyu_nn with tran_black_close()
    旁白 "我穿着围裙，端出了炒土豆丝和番茄蛋汤。"
    旁白 "单人桌挤了两个人。"
    show hongye pajamas_normal2 at s11
    子文 "寒舍狭小，招待不周，还请红叶大小姐见谅。"
    play music "audio/BGM/4.終幕への間奏曲.mp3"
    旁白 "红叶看了一眼我做的菜，走到了厨房里。红叶打开了冰箱。"
    红叶 pajamas_normal3 "唔…"
    旁白 "然后她看见了泡面盒。"
    子文 "呃…"
    红叶 pajamas_normal1 "原来我们的大作家子文君平时就吃这些吗？"
    子文 "这段时间没有什么时间跑菜市场了，这样解决倒也方便，还省的我洗碗。"
    红叶 pajamas_backhand2 "就算是这样也不会只买土豆、番茄和鸡蛋的。子文君大概只会这两道菜吧？"
    子文 "呃，因为…因为好弄好吃又下饭嘛。"
    旁白 "红叶笑了。她尝了一口菜，我极度紧张地看着她。"
    红叶 pajamas_normal1 "嗯，味道还可以啊。"
    子文 "感…感谢红叶大小姐赞赏。"
    红叶 pajamas_backhand1 "这样吧，明天开始，我来做饭，就当借住的补偿了，如何？"
    子文 "啊？"
    红叶 pajamas_backhand4 "不行吗？你不会忍心让我睡大街吧？"
    子文 "不是不是，红叶大小姐当然可以随意住下，就是…"
    红叶 pajamas_backhand4 "哟，看起来不太情愿啊，小时候我们可是经常睡一张床的哦。我妈还留了照片的…"
    子文 "得得得，别找别找。我倒是不介意，主要是红叶你这样大晚上还下着雨在外面一个人到处走，可不得把你爸妈和老师都急死了。"
    红叶 pajamas_backhand3 "那我有什么办法嘛…"
    子文 "我是觉得也不用…"
    stop music
    show hongye pajamas_normal2 at s22
    play sound "<to 1.0>audio/SE/14.电话（急促电话铃声）.wav"
    旁白 "这时我的电话响了。在我反应之前，红叶直接去接了起来，打了一个手势示意我不要出声。"
    #这里可以拆成多句，再口语化一点
    红叶 pajamas_normal2 "{cps=20}喂，妈，是我，红叶，{w=1.0}对是我，您别急，喘口气。"
    红叶 "{cps=20}我现在在子文家，很安全，不要担心。但是请您务必不要来找我，也不要再找子文。否则我立刻去睡桥洞。别的事情请不要现在讲，我们都需要冷静一下…"
    play sound "audio/SE/挂电话.mp3"
    旁白 "红叶把电话挂断。她的声音已经有些沙哑。"
    show hongye pajamas_backhand6 at s11
    子文 "红叶，你…"
    红叶 pajamas_normal2 "他们似乎不打算把我的话当话，那我也没必要听他们说完。感觉还是子文做的饭更让我开心。"
    hide hongye with dissolve
    scene bg ziwengongyuguanxishi_n with tran_black_anticlockwise()
    旁白 "吃完饭之后，我去洗我还有红叶的衣服。然后我就碰到了问题。"
    play music "audio/BGM/12.なんでしょう？.mp3"
    子文 "呃…"
    旁白 "我还是头一次自己手洗…异性的内衣。"
    旁白 "有些难堪。"
    子文 "红叶？"
    旁白 "没有回应。"
    子文 "红叶？"
    旁白 "我加大了声音，但是仍然没有回应。"
    子文 "行吧…"
    stop music
    scene bg ziwendegongyu_n with tran_black_close()
    show hongye pajamas_normal3 at s11
    子文 "我洗完衣服走出来的时候，红叶正坐在书桌前翻看着我的参考书和教科书。"
    子文 "衣服都已经烘干过了，明天应该可以直接穿…"
    红叶 pajamas_normal3 "哈，这书是全新的啊，你不会完全没翻过吧？连习题册都是全新的？"
    play music "audio/BGM/12.なんでしょう？.mp3"
    子文 "呃…"
    红叶 pajamas_backhand2 "那不会你整整一年来的作业都是抄的吧？"
    子文 "…"
    旁白 "我挠头。"
    红叶 pajamas_backhand3 "我的天哪，你这样，哎，还有一个月就要考试了啊，你真的还能弄得完这些知识点吗？"
    子文 "别骂了别骂了我的红叶奶奶。"
    show hongye pajamas_backhand2 at s11("close")
    红叶 pajamas_backhand2 "你过来，我给你慢慢讲。"
    子文 "饶了我吧，我还没预习到这里呢…"
    hide hongye with dissolve
    旁白 "…"
    stop music
    #TODO:你这深夜和夜怎么是一张图啊
    #FIXME:美工组想想办法
    scene bg ziwendegongyu_nn with dissolve
    旁白 "深夜。"
    旁白 "红叶睡床，我自己则抱着被子准备睡在我的小沙发上。"
    旁白 "我把书桌的椅子搬过来才勉强让我躺得下。"
    show hongye pajamas_normal4 at s11
    camera:
        blur 1
        matrixcolor BrightnessMatrix(-0.05)
    红叶 pajamas_normal4 "子文？"
    camera:
        blur 2
        matrixcolor BrightnessMatrix(-0.1)
    子文 "无边落木萧萧下…" with dissolve
    camera:
        blur 3
        matrixcolor BrightnessMatrix(-0.15)
    红叶 pajamas_backhand1 "子文君？"
    camera:
        blur 4
        matrixcolor BrightnessMatrix(-0.2)
    子文 "虚拟语气变形规则…" with dissolve
    camera:
        blur 5
        matrixcolor BrightnessMatrix(-0.25)
    红叶 pajamas_backhand3 "子~文~"
    camera:
        blur 6
        matrixcolor BrightnessMatrix(-0.3)
    子文 "氧化还原反应…" with dissolve
    camera:
        blur 7
        matrixcolor BrightnessMatrix(-0.35)
    红叶 pajamas_normal3 "哼…"
    camera:
        blur 0
        matrixcolor BrightnessMatrix(0.0)
    play sound "audio/SE/punch.mp3"
    show hongye with hpunch
    旁白 "我被一个靠枕砸了，立刻醒了过来。"
    子文 "喂，你搞什么？"
    红叶 pajamas_backhand1 "你过来睡吧。挤挤总归比沙发那别扭地方舒服。"
    子文 "不要。"
    红叶 pajamas_backhand3 "不要？"
    子文 "不要。小时候红叶睡觉就喜欢踢人，有一回不就把我踹下去了嘛。"
    红叶 pajamas_normal3 "你…怎么英语单词记不住，这种事情倒是这么清楚？"
    子文 "你要是被踹下去一回你也会记得的。"
    红叶 pajamas_backhand1 "姆…赶快过来，你这样明天指定全身酸的去不了学校。"
    子文 "…"
    hide hongye with dissolve
    旁白 "于是我搬着被子睡到了红叶身边。二人背对，耳边只有空调机的声音，淅淅沥沥的雨声，还有若有若无的喘息。"
    play music "audio/BGM/7.街を歩けば.mp3"
    show hongye pajamas_normal1 at s11("close")
    红叶 pajamas_normal1 "啊，感觉回到了十几年前呢。"
    子文 "很多事情都变了哦。你也变了，我也变了，世界也在变。那个时候的声优行业，和现在几乎是天差地别了哦。"
    红叶 pajamas_normal2 "唉…"
    子文 "所以，你还是打算坚持？"
    红叶 pajamas_backhand2 "当然咯。我从来都没有动摇过。我每天都在精进实力，而那些劝我放弃的人的的说辞倒是始终如一。"
    红叶 "真是的，明明根本就不了解这个行业，甚至不想去了解，就要凭片面的刻板印象劝别人放弃。"
    子文 "你获得湖城玉樱赏的事情，他们大概还不知道吧？"
    红叶 pajamas_normal1 "肯定不知道。他们眼里我还只是个因为兴趣就冲动做决定的高中生。作为落英的我的身份可是藏得很好的，那么浓的妆不是白化的哦。"
    子文 "或许，你可以试一试把这个事情说出去，他们如果看到了你的实力和你的成就，或许就会认可你了。"
    红叶 pajamas_backhand2 "拉倒吧，他们才不会懂得这里面的辛苦和技术。"
    子文 "不要这样想嘛。我也对声优一无所知，但是我的耳朵是诚实的，还是会为美妙的声音倾倒的啊。"
    红叶 pajamas_normal5 "哦，是这样吗？"
    子文 "？"
    hide hongye with dissolve
    旁白 "红叶转过来，开始对我的耳朵吹气。"
    子文 "喂，这又是搞什么？"
    红叶 pajamas_backhand5 "嘿嘿，耳朵还是诚实的嘛。这是为子文特供的添寝ASMR哦。"
    子文 "好了好了你赶快睡觉吧，明明已经这么久没有睡觉了，怎么这会这么精神啊？"
    红叶 pajamas_backhand4 "不要酬劳的ASMR都不要嘛。"
    旁白 "红叶转了回去。"
    子文 "你这样搞我还怎么睡得着啊啊啊…"
    旁白 "她笑起来。"
    stop music
    旁白 "…"
    旁白 "房间里归于安静，大概红叶已经睡着了吧。"
    旁白 "我起身确认了门窗上锁，然后走回了床边。"
    play music "audio/BGM/13.ゆりかご.mp3"
    旁白 "红叶看起来睡得很沉。白天，还是发生了很多事情啊。"
    子文 "辛苦了，我的大小姐。"
    旁白 "不知怎么，感觉有点羡慕，很羡慕红叶能有一件能让她为之付出一切的事情。"
    旁白 "而我仍然是那个我，不务正业，一事无成，甚至连前进的方向都不知道。没变的只有我自己啊。"
    旁白 "我为她盖好了被子，把每一个角都好好地塞实了。"
    子文 "淋了那么大的雨，可不能再受凉了。"
    旁白 "我躺回了床上。"
    子文 "确实，很多事情都变了呢…"
    旁白 "意识逐渐下沉了。"
    stop music
    scene black with tran_uf()
    pause 0.5
    ###小学的教室##TODO:是不是有必要弹出标签展示章节和地点名称
    scene bg xiaoxuejiaoshi_ at blured with dissolve
    scene bg xiaoxuejiaoshi_ at antiblur
    pause 0.8
    #FIXME: 改了，jz有空做一下章节效果，别的地方也能用
    旁白 "似乎是小学的教室..."
    play music "audio/BGM/3.語られざる過去.mp3"
    子文 "红叶，你真的没事吗？"
    小学红叶 "我没事。稍微有点感冒而已。"
    子文 "你要不还是去请个假吧，这样去考试的话，恐怕会更严重啊。"
    小学红叶 "没关系的。这有什么大不了的事情。"
    旁白 "我摸了摸她的头。"
    子文 "你这是发烧了啊，快回去休息。"
    小学红叶 "子文怎么比我还矫情啊？"
    子文 "…"
    scene bg xiaoxuejiaoshi_y with tran_black_anticlockwise()
    旁白 "考完之后，红叶就一直趴在桌子上。我立刻去找老师。"
    子文 "老师，红叶她生病了！"
    旁白 "…"
    旁白 "红叶被接走去医院了。"
    scene black with dissolve
    旁白 "…"
    旁白 "结果那一次考试红叶还是考了满分。"
    #故意的，不是bug
    scene black with Pause(1.0)
    stop music
     
