pub struct Courses;

pub mod courses {
    #![allow(dead_code)]

    pub const OPERATION_NAME: &'static str = "Courses";
    pub const QUERY: &'static str = "query Courses {\n    allCourses {\n        _id\n        name\n    }\n}\n\nquery Modules($course_id: ID!) {\n    course(id: $course_id) {\n        id\n        name\n        modulesConnection {\n            nodes {\n                _id\n                name\n                moduleItems {\n                    _id\n                    url\n                    content {\n                        __typename\n                        ... on File {\n                            _id\n                            displayName\n                            contentType\n                            createdAt\n                            updatedAt\n                        }\n                    }\n                }\n            }\n            pageInfo {\n                hasNextPage\n            }\n        }\n    }\n}\n";

    use serde::{Deserialize, Serialize};

    #[allow(dead_code)]
    type Boolean = bool;
    #[allow(dead_code)]
    type Float = f64;
    #[allow(dead_code)]
    type Int = i64;
    #[allow(dead_code)]
    type ID = String;

    #[derive(Deserialize)]
    pub struct CoursesAllCourses {
        #[doc = "legacy canvas id"]
        #[serde(rename = "_id")]
        pub id: ID,
        pub name: String,
    }

    #[derive(Serialize)]
    pub struct Variables;

    #[derive(Deserialize)]
    pub struct ResponseData {
        #[doc = "All courses viewable by the current user"]
        #[serde(rename = "allCourses")]
        pub all_courses: Option<Vec<CoursesAllCourses>>,
    }
}

impl graphql_client::GraphQLQuery for Courses {
    type Variables = courses::Variables;
    type ResponseData = courses::ResponseData;
    fn build_query(variables: Self::Variables) -> ::graphql_client::QueryBody<Self::Variables> {
        graphql_client::QueryBody {
            variables,
            query: courses::QUERY,
            operation_name: courses::OPERATION_NAME,
        }
    }
}

pub struct Modules;

pub mod modules {
    #![allow(dead_code)]

    pub const OPERATION_NAME: &'static str = "Modules";
    pub const QUERY: &'static str = "query Courses {\n    allCourses {\n        _id\n        name\n    }\n}\n\nquery Modules($course_id: ID!) {\n    course(id: $course_id) {\n        id\n        name\n        modulesConnection {\n            nodes {\n                _id\n                name\n                moduleItems {\n                    _id\n                    url\n                    content {\n                        __typename\n                        ... on File {\n                            _id\n                            displayName\n                            contentType\n                            createdAt\n                            updatedAt\n                        }\n                    }\n                }\n            }\n            pageInfo {\n                hasNextPage\n            }\n        }\n    }\n}\n";

    use serde::{Deserialize, Serialize};

    #[allow(dead_code)]
    type Boolean = bool;
    #[allow(dead_code)]
    type Float = f64;
    #[allow(dead_code)]
    type Int = i64;
    #[allow(dead_code)]
    type ID = String;
    #[doc = "an ISO8601 formatted time string"]
    type DateTime = super::DateTime;
    type URL = super::URL;

    #[derive(Deserialize)]
    pub struct ModulesCourseModulesConnectionNodesModuleItemsContentOnFile {
        #[doc = "legacy canvas id"]
        #[serde(rename = "_id")]
        pub id: ID,
        #[serde(rename = "displayName")]
        pub display_name: Option<String>,
        #[serde(rename = "contentType")]
        pub content_type: Option<String>,
        #[serde(rename = "createdAt")]
        pub created_at: Option<DateTime>,
        #[serde(rename = "updatedAt")]
        pub updated_at: Option<DateTime>,
    }

    #[derive(Deserialize)]
    #[serde(tag = "__typename")]
    pub enum ModulesCourseModulesConnectionNodesModuleItemsContentOn {
        File(ModulesCourseModulesConnectionNodesModuleItemsContentOnFile),
        Assignment,
        ExternalTool,
        ModuleExternalTool,
        Page,
        SubHeader,
        Quiz,
        Discussion,
        ExternalUrl,
    }

    #[derive(Deserialize)]
    pub struct ModulesCourseModulesConnectionNodesModuleItemsContent {
        #[serde(flatten)]
        pub on: ModulesCourseModulesConnectionNodesModuleItemsContentOn,
    }

    #[derive(Deserialize)]
    pub struct ModulesCourseModulesConnectionNodesModuleItems {
        #[doc = "legacy canvas id"]
        #[serde(rename = "_id")]
        pub id: ID,
        pub url: Option<URL>,
        pub content: Option<ModulesCourseModulesConnectionNodesModuleItemsContent>,
    }

    #[derive(Deserialize)]
    pub struct ModulesCourseModulesConnectionNodes {
        #[doc = "legacy canvas id"]
        #[serde(rename = "_id")]
        pub id: ID,
        pub name: Option<String>,
        #[serde(rename = "moduleItems")]
        pub module_items: Option<Vec<ModulesCourseModulesConnectionNodesModuleItems>>,
    }

    #[derive(Deserialize)]
    #[doc = "Information about pagination in a connection."]
    pub struct ModulesCourseModulesConnectionPageInfo {
        #[doc = "When paginating forwards, are there more items?"]
        #[serde(rename = "hasNextPage")]
        pub has_next_page: Boolean,
    }

    #[derive(Deserialize)]
    #[doc = "The connection type for Module."]
    pub struct ModulesCourseModulesConnection {
        #[doc = "A list of nodes."]
        pub nodes: Option<Vec<Option<ModulesCourseModulesConnectionNodes>>>,
        #[doc = "Information to aid in pagination."]
        #[serde(rename = "pageInfo")]
        pub page_info: ModulesCourseModulesConnectionPageInfo,
    }

    #[derive(Deserialize)]
    pub struct ModulesCourse {
        pub id: ID,
        pub name: String,
        #[serde(rename = "modulesConnection")]
        pub modules_connection: Option<ModulesCourseModulesConnection>,
    }

    #[derive(Serialize)]
    pub struct Variables {
        pub course_id: ID,
    }

    impl Variables {}

    #[derive(Deserialize)]
    pub struct ResponseData {
        pub course: Option<ModulesCourse>,
    }
}

impl graphql_client::GraphQLQuery for Modules {
    type Variables = modules::Variables;
    type ResponseData = modules::ResponseData;
    fn build_query(variables: Self::Variables) -> ::graphql_client::QueryBody<Self::Variables> {
        graphql_client::QueryBody {
            variables,
            query: modules::QUERY,
            operation_name: modules::OPERATION_NAME,
        }
    }
}
