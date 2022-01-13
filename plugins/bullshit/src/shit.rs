use crate::BullshitPlugin;
use countdown_bot3::countdown_bot::{client::ResultType, command::SenderType};
use jieba_rs::Jieba;
use rand::prelude::SliceRandom;
// use rand::seq::SliceRandom;
impl BullshitPlugin {
    pub async fn command_shit(&self, sentence: &str, sender: &SenderType) -> ResultType<()> {
        let jieba = Jieba::new();
        let mut words = jieba.cut(sentence, false);
        {
            let mut rng = rand::thread_rng();
            words.shuffle(&mut rng);
        }
        self.client
            .clone()
            .unwrap()
            .quick_send_by_sender(sender, &words.join(""))
            .await?;
        Ok(())
    }
}
