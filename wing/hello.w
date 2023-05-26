bring cloud;

// API requests get pushed to a queue ...
let api = new cloud.Api();
let queue = new cloud.Queue();
api.get("/", inflight(req: cloud.ApiRequest): cloud.ApiResponse => {

    let data = Json.stringify(req);
    log(data);
    queue.push(data);

    return cloud.ApiResponse {
        status: 200,
        body: "Thanks for the request!"
    };
});

// And then taken from the queue and stored into a table, and an S3 bucket
let table = new cloud.Table(cloud.TableProps{
  name: "requests",
  primaryKey: "id",
  columns: {
    "request_body": cloud.ColumnType.STRING
  }
});
let bucket = new cloud.Bucket();

// Update the bucket
queue.addConsumer(inflight (message: str) => {
  bucket.put("lastrequest.txt", message);
}, cloud.QueueProps {
  
});

// Update the table
// queue.addConsumer(inflight (message: str) => {
//   table.insert("lastrequest", message);
// });
