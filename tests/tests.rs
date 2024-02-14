mod pic {
  mod nested_pic;
  mod simple;
}

mod visual {
  mod diagram;
  mod edges;
  mod hello;
  mod units;
}

mod doc {
  mod align;
  mod edges;
}

mod diagram;

#[macro_export]
macro_rules! assert_diagram {
  ($diagram:expr) => {
    fn stub() {}
    fn type_name_of<T>(_: T) -> &'static str {
        std::any::type_name::<T>()
    }

    let function_name = type_name_of(stub).rsplit("::").collect::<Vec<_>>();

    let path = std::path::Path::new(file!());
    let stem = path.parent().map(|parent| parent.join(path.file_stem().unwrap())).unwrap();

    let prefix = format!("{}-{}", stem.to_str().unwrap(), function_name[1]);
    assert_diagram($diagram, &prefix).unwrap();
  };
}

