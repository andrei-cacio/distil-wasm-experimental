const js = import("./web/distil");

js.then(js => {
  console.log(js);
  console.log(js.concat('hello', 'wolrd'));
});