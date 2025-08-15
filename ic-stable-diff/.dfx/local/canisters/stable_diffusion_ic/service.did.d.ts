import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';
import type { IDL } from '@dfinity/candid';

export interface ApiResponse {
  'data' : [] | [string],
  'error' : [] | [string],
  'timestamp' : bigint,
  'success' : boolean,
}
export interface ApiResponseImage {
  'data' : [] | [Uint8Array | number[]],
  'error' : [] | [string],
  'timestamp' : bigint,
  'success' : boolean,
}
export interface ApiResponseTask {
  'data' : [] | [GenerationTask],
  'error' : [] | [string],
  'timestamp' : bigint,
  'success' : boolean,
}
export interface GenerationRequest {
  'height' : [] | [number],
  'seed' : [] | [bigint],
  'num_inference_steps' : [] | [number],
  'prompt' : string,
  'width' : [] | [number],
  'guidance_scale' : [] | [number],
  'negative_prompt' : [] | [string],
}
export interface GenerationTask {
  'id' : string,
  'status' : TaskStatus,
  'result' : [] | [Uint8Array | number[]],
  'request' : GenerationRequest,
  'created_at' : bigint,
  'error' : [] | [string],
  'completed_at' : [] | [bigint],
}
export type TaskStatus = { 'Failed' : null } |
  { 'Processing' : null } |
  { 'Completed' : null } |
  { 'Pending' : null };
export interface _SERVICE {
  'generate_image' : ActorMethod<[GenerationRequest], ApiResponse>,
  'get_image' : ActorMethod<[string], ApiResponseImage>,
  'get_task_status' : ActorMethod<[string], ApiResponseTask>,
  'http_request' : ActorMethod<
    [
      {
        'url' : string,
        'method' : string,
        'body' : Uint8Array | number[],
        'headers' : Array<[string, string]>,
      },
    ],
    {
      'body' : Uint8Array | number[],
      'headers' : Array<[string, string]>,
      'streaming_strategy' : [] | [
        {
            'Callback' : {
              'token' : [] | [Uint8Array | number[]],
              'callback' : [Principal, string],
            }
          }
      ],
      'status_code' : number,
    }
  >,
  'list_tasks' : ActorMethod<
    [],
    {
      'data' : [] | [Array<string>],
      'error' : [] | [string],
      'timestamp' : bigint,
      'success' : boolean,
    }
  >,
}
export declare const idlFactory: IDL.InterfaceFactory;
export declare const init: (args: { IDL: typeof IDL }) => IDL.Type[];
