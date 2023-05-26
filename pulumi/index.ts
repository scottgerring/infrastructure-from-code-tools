import * as pulumi from "@pulumi/pulumi";
import * as aws from "@pulumi/aws";
import * as awsx from "@pulumi/awsx";
import * as apigateway from "@pulumi/aws-apigateway";

import { randomUUID } from "crypto";

// Create an S3 bucket and a dynamoDB table we'll write our
// data to. 
const bucket = new aws.s3.Bucket("my-bucket");
const table = new aws.dynamodb.Table("my-table", {
  attributes: [
    { name: "id", type: "S" },
  ],
  hashKey: "id",
  billingMode: "PAY_PER_REQUEST"
})

// Create a queue and handler that'll do the actual writing
const queue = new aws.sqs.Queue("my-queue", {
    visibilityTimeoutSeconds: 200
});


// Event handler to write to the bucket
queue.onEvent("message-to-bucket", async (msg) => {
    let rec = msg.Records[0];
    let id = rec.messageId;

    const s3 = new aws.sdk.S3();
    await s3.putObject({
        Bucket: bucket.bucket.get(),
        Key: id,
        Body: rec.body
    }).promise();
}, {
    batchSize: 1
});

// Event handler to write to the table
queue.onEvent("message-to-table", async (msg) => {
  let rec = msg.Records[0];
  let id = rec.messageId;

  const ddb = new aws.sdk.DynamoDB.DocumentClient();
  await ddb.put({
    TableName: table.name.get(),
    Item: {
      id: id,
      request: rec.body
    }
  }).promise();  

}, {
  batchSize: 1
});


// create an API to hit to write to the queue
const api = new apigateway.RestAPI("api", {
    routes: [
      {
        path: "/",
        method: "GET",
        eventHandler: new aws.lambda.CallbackFunction("api-handler", {
          callback: async (evt: any, ctx) => {            
            const sqs = new aws.sdk.SQS();
            
            await sqs.sendMessage({
              QueueUrl: queue.url.get(),
              MessageBody: JSON.stringify(evt)
            }).promise();

            return {
              statusCode: 200,
              body: JSON.stringify({
                "message:" : "Hello, API Gateway!",
            })}
          }                
        })
      }
    ]
  });


// Export the name of the bucket
export const bucketName = bucket.id;
export const apiUrl = api.url;
