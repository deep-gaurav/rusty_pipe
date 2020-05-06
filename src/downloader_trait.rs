use async_trait::async_trait;

#[async_trait]
pub trait Downloader{

    async fn download(&self,url:&str)->Result<String,String>;

}