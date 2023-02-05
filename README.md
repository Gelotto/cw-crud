# CosmWasm "Repository" Smart Contract

This is a CosmWasm implementation of the [_repository design pattern_](https://deviq.com/design-patterns/repository-pattern).

> Essentially, it provides an abstraction of data, so that your application can work with a simple abstraction that has an interface approximating that of a collection. Adding, removing, updating, and selecting items from this collection is done through a series of straightforward methods, without the need to deal with database concerns like connections, commands, cursors, or readers. Using this pattern can help achieve loose coupling and can keep domain objects persistence ignorant.

If your app is entirely on-chain and you are not using a database to store and contract addresses or state, this contract is for you. It lets you request and paginate collections of other contracts instantiated through it. It gives you efficient indexing, ordering, and pagination and a simple client to use from within other smart contracts.

## Example Use Case

Imagine an app in which each user is given their own Account contract. This app could instantiate each Account through a Repository. Now we can query the Repository for all Accounts, say 10 at a time, that are active and have been updated in the last 24 hours. This greatly simplifies the task of building a page in the app that displays an scrolling grid of most recently active users.

## Features

1. Pagination over indexed collection of contracts.
1. Batch execution of contracts matched by queries.
1. Batch deletion of contracts matched by queries.
1. String, numeric, boolean, and timestamp indexes on custom fields.
1. Built-in indexes for the following:
   - Creation Timestamp
   - Update Timestamp
   - Revision number
   - Address
   - Code ID

## Adding Contracts to a Repository

Contracts that are instantiated by a repo's `create` function are automatically added to the collection and indexed. Indexed fields must be explicitly specified. Here's an example of instantiating a simple integer "counter" smart contract through a repo:

```typescript
const initialCounterValue = 0;
const instantiate_msg = { count: initialCounterValue };

await client.execute(
    senderAddress,
    repoAddress,
    {
        create: {
            code_id,
            instantiate_msg,
            // add initial value to an index on the `count` field
            // of the instantiated contract. each custom index is
            // identified by an string name:
            indices: [{
                numeric: {
                    slot: 0,
                    value: initialCounterValue
                }
            }]
        }
    })
);
```

## Paginatated Queries

One there are a few contracts in a repo, you can query them and paginate the results via the `select` function. The results of a select consist of at least each contract address matched by the query but can also include metadata about each contract as well as state returned from each contract itself. To caching purposes, it is possible to prune the returned results to include only contracts that have been modified since a given block time or revision number.

Continuing with the example above, suppose you'd like to fetch the first 20 Counter contracts that have been updated at least once, returning their stored counters (i.e. their `count` state), ordered from greatest to least count. We could do:

```typescript

const result = await client.queryContractSmart(repoAddress, {
  select: {
    index: { numeric: { slot: 0 } } },
    fields: ["count"],
    since: { rev: 1 }
    desc: true,
    limit: 20,
  },
});
```
