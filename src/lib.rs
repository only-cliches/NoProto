use json::*;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}


pub struct NoProtoDataModel {
    key: String,
    colType: String,
    options: JsonValue,
    columns: Box<Vec<NoProtoDataModel>>, // nested table
    arrayType: Box<Option<NoProtoDataModel>> // nested array
}

pub struct NoProtoFactory {
    dataModel: Box<NoProtoDataModel>
}

pub struct NoProtoBuffer<'a> {
    factory: &'a NoProtoFactory,
    bytes: Box<Vec<u8>>,
    ptr: usize
}

impl NoProtoFactory {

    pub fn load_model_from_object(&mut self, model: JsonValue) {

        self.dataModel = Box::new(NoProtoDataModel {
            key: "root".to_string(),
            colType: "root".to_string(),
            options: object!{},
            columns: self.load_data_columns(model.clone()),
            arrayType: Box::new(None)
        });
    }

    fn load_data_columns(&self, model: JsonValue) -> Box<Vec<NoProtoDataModel>> {

        let mut i: usize = 0;

        let length = model.len();

        let mut columns = vec![];

        loop {
            if i < length {

                let model_row: &JsonValue = &model[i];

                let row_key = &model_row[0].as_str().unwrap_or("").to_owned();
                let row_type = &model_row[1].as_str().unwrap_or("").to_owned();
                let row_options = if model_row[2].is_null() { object!{} } else { model_row[2].clone() };

                match row_type.rfind("[]") {
                    Some(x) => {
                        let this_type = &row_type[0..x];
                        columns.push(self.load_data_array(this_type.to_owned()));
                    },
                    None => {
                        columns.push(NoProtoDataModel {
                            key: row_key.to_string(),
                            colType: row_type.to_owned(),
                            columns: if row_options["model"].is_null() { Box::new(vec![]) } else { self.load_data_columns(row_options["model"].clone()) },
                            options: row_options,
                            arrayType: Box::new(None)
                        });
                    }
                }

                i += 1;
            } else {
                break;
            }
        }

        return Box::new(columns);
    }

    fn load_data_array(&self, col_type: String) -> NoProtoDataModel {

        NoProtoDataModel {
            key: "".to_string(),
            colType: "List".to_string(),
            columns: Box::new(vec![]),
            options: object!{},
            arrayType: match col_type.rfind("[]") {
                Some(x) => {
                    let this_type = &col_type[0..x];
                    Box::new(Some(self.load_data_array(this_type.to_owned())))
                },
                None => Box::new(None)
            }
        }
    }

    pub fn load_model_from_string(&mut self, model: &str) {
        self.load_model_from_object(json::parse(model).unwrap());
    }

    pub fn new_buffer(&self) -> NoProtoBuffer {
        NoProtoBuffer {
            factory: self,
            bytes: Box::new(vec![]),
            ptr: 0
        }
    }

    pub fn parse_buffer(&self, in_buffer: Vec<u8>) -> NoProtoBuffer {
        NoProtoBuffer {
            factory: self,
            ptr: in_buffer.len(),
            bytes: Box::new(in_buffer),
        }
    }
}