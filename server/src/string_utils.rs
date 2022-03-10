/*
 * Copyright 2022 nzelot<leontsteiner@gmail.com>
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 * http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

pub trait UnifyQuotes {
    fn unify_quotes(&self) -> String;
}

pub trait UnifyApostrophes {
    fn unify_apostrophes(&self) -> String;
}

impl UnifyQuotes for str {
    fn unify_quotes(&self) -> String {
        replace_quotes(self)
    }
}

impl UnifyQuotes for String {
    fn unify_quotes(&self) -> String {
        replace_quotes(self)
    }
}

impl UnifyApostrophes for str {
    fn unify_apostrophes(&self) -> String {
        replace_apostrophe(self)
    }
}

impl UnifyApostrophes for String {
    fn unify_apostrophes(&self) -> String {
        replace_apostrophe(self)
    }
}

fn replace_quotes(input : &str) -> String {
    input.replace("“", "\"")
        .replace("”", "\"")
}

fn replace_apostrophe(input : &str) -> String {
    input.replace("’", "'")
        .replace("`", "'")
        .replace("´", "'")
}