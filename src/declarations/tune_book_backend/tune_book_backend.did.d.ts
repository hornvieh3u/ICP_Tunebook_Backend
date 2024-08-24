import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';
import type { IDL } from '@dfinity/candid';

export interface Friend {
  'principal' : string,
  'username' : string,
  'avatar' : Uint8Array | number[],
}
export interface OriginTune { 'title' : string, 'tune_data' : string }
export interface Profile {
  'pob' : string,
  'principal' : string,
  'username' : string,
  'incoming_fr' : Array<Friend>,
  'outcoming_fr' : Array<Friend>,
  'instruments' : string,
  'friends' : Array<string>,
  'avatar' : Uint8Array | number[],
}
export interface Tune {
  'id' : number,
  'title' : string,
  'thumbnail' : Uint8Array | number[],
  'origin' : boolean,
  'timestamp' : bigint,
  'tune_data' : [] | [string],
}
export interface UserTune {
  'id' : number,
  'title' : string,
  'thumbnail' : Uint8Array | number[],
}
export interface _SERVICE {
  'accept_friend_request' : ActorMethod<[string, string], boolean>,
  'add_tune' : ActorMethod<
    [string, string, string, boolean, Uint8Array | number[]],
    boolean
  >,
  'authentication' : ActorMethod<[string], [] | [Profile]>,
  'browse_people' : ActorMethod<[string, number], [Array<Friend>, number]>,
  'filter_tunes' : ActorMethod<
    [string, string, string, number],
    [Array<OriginTune>, number]
  >,
  'get_friends' : ActorMethod<[string], Array<Friend>>,
  'get_new_tunes_from_friends' : ActorMethod<[string], Array<Tune>>,
  'get_original_tune' : ActorMethod<[string], string>,
  'get_original_tune_list' : ActorMethod<[number], [Array<string>, number]>,
  'get_user_tune' : ActorMethod<[string, string], string>,
  'get_user_tune_list' : ActorMethod<
    [string, number],
    [Array<UserTune>, number]
  >,
  'init' : ActorMethod<[], undefined>,
  'send_friend_request' : ActorMethod<[string, string], [] | [Friend]>,
  'update_profile' : ActorMethod<
    [string, string, string, string, Uint8Array | number[]],
    Profile
  >,
  'update_tune' : ActorMethod<
    [number, string, string, string, boolean, Uint8Array | number[]],
    boolean
  >,
}
export declare const idlFactory: IDL.InterfaceFactory;
export declare const init: (args: { IDL: typeof IDL }) => IDL.Type[];
