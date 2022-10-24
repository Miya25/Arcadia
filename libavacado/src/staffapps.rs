use sqlx::PgPool;

use crate::types::{StaffAppData, StaffPosition, StaffAppQuestion, Error, StaffAppResponse};

pub fn get_interview_questions() -> Vec<StaffAppQuestion> {
	vec![
		StaffAppQuestion {
			id: "motive".to_string(),
			question: "Why did you apply for this role?".to_string(),
			para: "Why did you apply for this role? Be specific. We want to know how you can help Infinity Bot List and why you wish to".to_string(),
			placeholder: "I applied because...".to_string(),
		},
		StaffAppQuestion {
			id: "team-player".to_string(),
			question: "What is a scenario in which you had to be a team player?".to_string(),
			para: "What is a scenario in which you had to be a team player? We want to know that you can collaborate effectively with us.".to_string(),
			placeholder: "I had to...".to_string(),
		},
		StaffAppQuestion {
			id: "some-work".to_string(),
			question: "What is some of the work you have done? Can you share some links with us?".to_string(),
			para: "What is some of the work you have done? Can you share some links with us? We want to see your finest works".to_string(),
			placeholder: "Some work I did...".to_string()
		},
		StaffAppQuestion {
			id: "about-you".to_string(),
			question: "Tell us a little about yourself".to_string(),
			para: "Tell us a little about yourself. Its that simple!".to_string(),
			placeholder: "I am...".to_string()
		}
	]
}

pub fn get_apps() -> StaffAppData {
    StaffAppData {
        positions: vec!["staff".to_string(), "dev".to_string()],
        staff: StaffPosition {
            open: true,
            name: "Staff Team".to_string(),
            info: r#"Join the Infinity Staff Team and help us Approve, Deny and Certify Discord Bots. 

We are a welcoming and laid back team who is always willing to give new people an opportunity!"#.to_string(),
            questions: vec![
                StaffAppQuestion {
                    id: "experience".to_string(),
                    question: "Past server experience".to_string(),
                    para: "Tell us any experience you have working for other servers or bot lists.".to_string(),
                    placeholder: "I have worked at...".to_string(),
                },
                StaffAppQuestion {
                    id: "strengths".to_string(),
                    question: "List some of your strengths".to_string(),
                    para: "Tell us a little bit about yourself.".to_string(),
                    placeholder: "I am always online and active...".to_string(),
                },
                StaffAppQuestion {
                    id: "situations".to_string(),
                    question: "Situation Examples".to_string(),
                    para: "How would you handle: Mass Pings, Nukes and Raids etc.".to_string(),
                    placeholder: "I would handle it by...".to_string(),
                },
                StaffAppQuestion {
                    id: "reason".to_string(),
                    question: "Why do you want to join the staff team?".to_string(),
                    para: "Why do you want to join the staff team? Be specific".to_string(),
                    placeholder: "I want to join the staff team because...".to_string(),
                },
                StaffAppQuestion {
                    id: "other".to_string(),
                    question: "Anything else you want to add?".to_string(),
                    para: "Anything else you want to add?".to_string(),
                    placeholder: "Just state anything that doesn't hit anywhere else".to_string(),
                },
            ],
        },
        dev: StaffPosition {
            open: true,
            name: "Dev Team".to_string(),
            info: r#"Join our Dev Team and help us update, manage and maintain all of the Infinity Services!
            
Experience in PostgreSQL and at least one of the below languages is required:

- Rust
- JavaScript
- Go
"#.to_string(),
            questions: vec![
                StaffAppQuestion {
                    id: "experience".to_string(),
                    question: "Do you have experience in Javascript, Rust and/or Golang. Give examples of projects/code you have written".to_string(),
                    para: "Do you have experience in Javascript, Rust and/or Golang. Give examples of projects/code you have written.".to_string(),
                    placeholder: "I have worked on...".to_string(),
                },
                StaffAppQuestion {
                    id: "strengths".to_string(),
                    question: "What are your strengths in coding Javascript, Rust and/or Golang.".to_string(),
                    para: "What are your strengths in coding Javascript, Rust and/or Golang.".to_string(),
                    placeholder: "I am good at...".to_string(),
                },
                StaffAppQuestion {
                    id: "db".to_string(),
                    question: "Do you have Exprience with PostgreSQL".to_string(),
                    para: "Do you have Exprience with PostgreSQL".to_string(),
                    placeholder: "I have used PostgreSQL for...".to_string(),
                },
                StaffAppQuestion {
                    id: "reason".to_string(),
                    question: "Why do you want to join the dev team?".to_string(),
                    para: "Why do you want to join the dev team? Be specific".to_string(),
                    placeholder: "I want to join the dev team because...".to_string(),
                },
            ]
        }
    }
}

pub async fn get_made_apps(pool: &PgPool) -> Result<Vec<StaffAppResponse>, Error> {
    let mut apps = Vec::new();

    let apps_db = sqlx::query!("SELECT user_id, created_at, state, answers, likes, dislikes, position FROM apps")
        .fetch_all(pool)
        .await?;
    
    for app in apps_db {
        let mut likes = Vec::new();

        for like in app.likes {
            likes.push(like.to_string());
        }

        let mut dislikes = Vec::new();

        for dislike in app.dislikes {
            dislikes.push(dislike.to_string());
        }

        apps.push(StaffAppResponse {
            user_id: app.user_id,
            created_at: app.created_at,
            state: app.state,
            answers: app.answers,
            position: app.position,
            likes,
            dislikes,
        });
    }
    
    Ok(vec![])
}
