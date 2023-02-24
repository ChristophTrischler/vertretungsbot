
use reqwest::{Client};
use scraper::{Html, Selector};
use serenity::builder::{CreateMessage, CreateEmbed};
use std::env;
use serde::{Deserialize, Serialize};
use chrono::naive::NaiveDate;
use prettytable::*;

pub async fn check_change(number: i64, last_time :&mut String)-> Option<VDay>{  
    let url = format!("https://geschuetzt.bszet.de/s-lk-vw/Vertretungsplaene/V_PlanBGy/V_DC_00{}.html",number);
    let c = Client::new();
    let res = c.get(url)
        .basic_auth("bsz-et-2223", Some(env::var("PW").expect("no PW in env")))
        .send()
        .await
        .unwrap();
    if !res.status().is_success() {
        return None;
    }
    let headers = res.headers();
    let this_time = headers.get("last-modified")
    .unwrap()
    .to_str()
    .unwrap();

    if last_time.as_str().eq(this_time) {
        return None;
    }
    else {
        *last_time = this_time.to_string();
    }
    let text = res.text()
    .await
    .unwrap();
    return Some(get_vday(&text));
}


fn is_in(string: &str, vec: &Vec<String>)->bool{
    for s in vec{
        if string.contains(s) {
            return true;
        }
    }
    return false;
}


pub async fn get_v_text(number: i64) -> Option<VDay>{
    let url = format!("https://geschuetzt.bszet.de/s-lk-vw/Vertretungsplaene/V_PlanBGy/V_DC_00{}.html",number);
    let c = Client::new();
    let res = c.get(url)
        .basic_auth("bsz-et-2223", Some(env::var("PW").expect("no PW in env")))
        .send()
        .await
        .unwrap();
    if !res.status().is_success() {
        return None;
    }
    return Some(get_vday(&res.text().await.unwrap()));
}

pub fn get_vday(text: &String) -> VDay {
    let doc = Html::parse_document(text);

    let date_selection = Selector::parse(r#"h1[class="list-table-caption"]"#).unwrap();
    let date = doc.select(&date_selection).next()
    .unwrap()
    .inner_html()
    .trim()
    .to_string();

    let table_body_selection = Selector::parse("tbody").unwrap();
    let table_row_selection = Selector::parse("tr").unwrap();
    let table_field_selection = Selector::parse("td").unwrap();

    let mut v_lessons: Vec<Lesson> = vec!();

    let table = doc.select(&table_body_selection).next().unwrap();

    for row in table.select(&table_row_selection){
        let fields = row.select(&table_field_selection);

        let mut content_fields:Vec<String> = fields.map(|item| item.inner_html().trim().to_string()).collect();
        if row.inner_html().contains("&nbsp;"){
            let last_lesson = v_lessons.last()
            .unwrap()
            .to_vec();
            for (i, s) in &mut content_fields.iter_mut()
            .enumerate()
            .filter(|(_i, s)| s.contains("&nbsp;")){
                let replacement = last_lesson.get(i).unwrap();
                *s = replacement.to_string();
            } 
        }
    
        v_lessons.push(Lesson{
            class: content_fields.get(0).unwrap().into(),
            time: content_fields.get(1).unwrap()
            .trim_end_matches('.')
            .parse()
            .unwrap(),
            subject: content_fields.get(2).unwrap().into(),
            room: content_fields.get(3).unwrap().into(),
            teacher: content_fields.get(4).unwrap().into(),
            vtype: content_fields.get(5).unwrap().into(),
            message: content_fields.get(6).unwrap().into()
        });
    }

    return VDay(date, v_lessons.into());
}


pub fn get_day(VDay(day_str, v_lessons): &VDay, plan :&Plan)->Day{
    let mut splits =  day_str.split_whitespace().into_iter();
    let day_name = splits.next().unwrap();
    let date_str =  splits.next().unwrap();

    let date = NaiveDate::parse_from_str(date_str, "%d.%m.%Y").unwrap();
    let ref_date = NaiveDate::from_ymd_opt(2022, 8, 22).unwrap();
    let week = date.signed_duration_since(ref_date).num_weeks() % 2;

    let mut res_day:Day = Day::new(&day_str.as_str());

    for v_lesson in v_lessons.iter().
    filter(|item| item.class.contains(&plan.class_name) && is_in(&item.subject, &plan.subjects)){
        res_day.lessons.get_mut((v_lesson.time-1) as usize)
        .unwrap()
        .push(v_lesson.clone());      
    }

    let plan_day = plan.days.iter()
    .find(|item| item.day.contains(day_name))
    .unwrap();
    let empty_times = res_day.lessons.iter_mut()
    .enumerate()
    .filter(|(i, item)| i%2==0 && item.len()==0);
 
    for (i, ls) in empty_times {
        let normal = plan_day.lessons.get(i/2).unwrap();
        match normal {
            WeekOption::AandB(l) => ls.push(l.to_lesson()),
            WeekOption::A(l) => if week == 1 {
                ls.push(l.to_lesson());
            },
            WeekOption::B(l) => if week == 2 {
                ls.push(l.to_lesson());
            },
            WeekOption::AorB(l1, l2) => if week == 1{
                ls.push(l1.to_lesson());
            }
            else {
                ls.push(l2.to_lesson());
            },
            WeekOption::None => (),   
        }   
    };
    return res_day; 
} 


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanLesson{
    time: i64,
    subject: String,
    room: String,
    teacher: String
}

impl PlanLesson {
    pub fn to_lesson(&self)->Lesson{
        Lesson::new(
            self.time,
            self.subject.as_str(),
            self.room.as_str(),
            self.teacher.as_str()
        )
    }
}


#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum WeekOption {
    #[default]
    None,
    AandB(PlanLesson),
    A(PlanLesson),
    B(PlanLesson),
    AorB(PlanLesson, PlanLesson),
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lesson {
    pub class: String,
    pub time: i64,
    pub subject: String,
    pub room: String,
    pub teacher: String,
    pub vtype: String, 
    pub message: String,
}

impl Lesson {
    fn new(time: i64, subject: &str, room: &str, teacher: &str)->Lesson{
        Lesson { 
            class: String::new(), 
            time: time, 
            subject: subject.to_string(), 
            room: room.to_string(), 
            teacher: teacher.to_string(), 
            vtype: String::new(), 
            message: String::new() 
        }
    }
    fn to_embed(&self, e: &mut CreateEmbed){
        let fields = vec![
            ("Fach",&self.subject,false),
            ("Raum",&self.room,false),
            ("Lehrer", &self.teacher,false),
            ("Art", &self.vtype,false),
            ("Mitteilung", &self.message,false)
        ].into_iter().filter(|i|i.1.len()>0);

        e.title(format!("{}.",self.time))
        .fields(fields);
    }

    fn to_row(&self) -> Row{
        row![self.time.to_string(),self.subject,self.room,self.teacher,self.vtype,self.message]
    }

    fn to_vec(&self) -> Vec<String>{
        vec![self.class.to_string(), self.time.to_string(), self.subject.to_string(), 
        self.room.to_string(), self.teacher.to_string(), self.vtype.to_string(), self.message.to_string()]
    }
}


pub struct VDay(String, Vec<Lesson>);


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Day {  
    pub day: String,
    pub lessons: [Vec<Lesson>; 10]
}

impl Day {
    pub fn new(day: &str) -> Day{
        Day {
            day: day.to_string(),
            lessons: Default::default()
        }
    }

    pub fn to_table(&self) -> Table{
        let mut table = Table::new();
        table.set_titles(row!["Stunde","Fach", "Raum","Lehrer","Type","Mitteilung"]);
        self.lessons.iter()
        .filter(|item| item.len()>0)
        .for_each(|lesson| 
            for l in lesson{
                table.add_row(l.to_row());
            }
        );
        table
    }

    pub fn to_embed(&self, m: &mut CreateMessage){
        m.content(&self.day);
        self.lessons.iter()
        .filter(|item| item.len()>0)
        .for_each(|lesson| 
            for l in lesson{
                m.add_embed(|e| {
                    l.to_embed(e);
                    e
                });
            }
        );
    }

    pub fn to_string(&self) -> String{
        format!{"```{}\n{}```",self.day, self.to_table().to_string()}
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct PlanDay{
    pub day: String, 
    pub lessons: [WeekOption; 5]
} 

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Plan {
    pub class_name: String,
    pub days: Vec<PlanDay>,
    pub subjects: Vec<String>
}