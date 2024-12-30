use super::ent;
use chrono::Utc;
use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use serde::Serialize;

#[derive(Serialize)]
struct EmailBody {
    update_time: String,
    events: Vec<ent::HealthInfo>,
}

pub struct Alarm {
    from: String,
    to: String,
    mailer: SmtpTransport,
}
impl Alarm {
    pub fn new(
        from: String,
        to: String,
        smtp_username: String,
        smtp_password: String,
        domain: String,
    ) -> Alarm {
        let creds = Credentials::new(smtp_username, smtp_password);
        Alarm {
            from,
            to,
            mailer: SmtpTransport::starttls_relay(&domain)
                .unwrap()
                .port(587)
                .credentials(creds)
                .build(),
        }
    }
    pub fn notify(&self, events: Vec<ent::HealthInfo>) {
        let body = EmailBody {
            update_time: Utc::now().to_rfc3339(),
            events,
        };
        let msg = serde_json::to_string_pretty(&body).unwrap();
        let email = Message::builder()
            .from(self.from.parse().unwrap())
            .to(self.to.parse().unwrap())
            .subject("资源监控预警")
            .header(ContentType::TEXT_PLAIN)
            .body(String::from(&msg))
            .unwrap();
        // Send the email
        match self.mailer.send(&email) {
            Ok(_) => tracing::info!("Email sent successfully!"),
            Err(e) => {
                tracing::error!("Could not send email: {e:?}");
                tracing::info!("Unsend mail: {:?}", &msg);
            }
        };
    }
}
#[cfg(test)]
mod tests {
    use lettre::message::header::ContentType;
    use lettre::transport::smtp::authentication::Credentials;
    use lettre::{Message, SmtpTransport, Transport};

    #[test]
    fn send_mail() {
        let email = Message::builder()
            .from("NoBody <nobody@example.com>".parse().unwrap())
            .to("Test <test@example.com>".parse().unwrap())
            .subject("Test email")
            .header(ContentType::TEXT_PLAIN)
            .body(String::from("Test message"))
            .unwrap();

        let username = std::env::var("SMTP_TEST_USERNAME").unwrap_or_else(|_| "test_user".to_string());
        let password = std::env::var("SMTP_TEST_PASSWORD").unwrap_or_else(|_| "test_pass".to_string());
        
        let creds = Credentials::new(username, password);

        // 使用测试 SMTP 服务器
        let mailer = SmtpTransport::relay("localhost")
            .unwrap()
            .credentials(creds)
            .build();

        // 仅在设置了测试环境变量时执行实际发送
        if std::env::var("RUN_SMTP_TEST").is_ok() {
            match mailer.send(&email) {
                Ok(_) => println!("Test email sent successfully!"),
                Err(e) => panic!("Could not send email: {e:?}"),
            };
        } else {
            println!("Skipping SMTP test - set RUN_SMTP_TEST env var to enable");
        }
    }
}
